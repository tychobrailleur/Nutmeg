use crate::error::NutmegError;
use crate::db::manager::DbManager;
use crate::service::auth::{AuthenticationService, HattrickAuthService};
use crate::service::secret::{SecretStorageService, SystemSecretService};
use crate::service::sync::{DataSyncService, ProgressCallback, SyncService};
use crate::ui::context_object::ContextObject;
use crate::ui::oauth_dialog::OAuthDialog;
use crate::window::NutmegWindow;
use gtk::glib;

use log::{error, info, warn};
use std::sync::Arc;

pub struct SyncController;

impl SyncController {
    /// Spawns a background task that fetches avatars and refreshes the squad view when done.
    fn spawn_avatar_refresh(
        db: Arc<DbManager>,
        context: ContextObject,
        key: String,
        secret: String,
        team_id: u32,
        download_id: i32,
    ) {
        let sync_clone = Arc::new(SyncService::new(db));
        let ctx_clone = context;
        glib::MainContext::default().spawn_local(async move {
            if let Err(e) = sync_clone
                .perform_avatar_sync_with_stored_secrets(key, secret, team_id, download_id)
                .await
            {
                warn!("Background avatar fetch failed: {}", e);
            } else {
                info!("Background avatar fetch completed. Refreshing player list.");
                if let Some(_selected) = ctx_clone.selected_team() {
                    ctx_clone.refresh_from_db();
                }
            }
        });
    }

    /// Spawns a background task that fetches archival matches for all teams in a series.
    pub fn spawn_series_form_refresh(
        db: Arc<DbManager>,
        context: ContextObject,
        key: String,
        secret: String,
        unit_id: i32,
        download_id: Option<i32>,
    ) {
        let sync_clone = Arc::new(SyncService::new(db.clone()));
        let ctx_clone = context.clone();
        glib::MainContext::default().spawn_local(async move {
            info!(
                "[sync] Spawning background form refresh for series {}",
                unit_id
            );

            let actual_download_id = match download_id {
                Some(id) => id,
                None => {
                    let db_inner = db.clone();
                    match tokio::task::spawn_blocking(move || {
                        let mut conn = db_inner.get_connection()?;
                        crate::db::download_entries::get_latest_download_id(&mut conn)
                            .map_err(|e| NutmegError::Db(e.to_string()))
                    })
                    .await
                    {
                        Ok(Ok(id)) => id,
                        _ => {
                            warn!("Failed to get latest download ID for background sync");
                            return;
                        }
                    }
                }
            };

            if let Err(e) = sync_clone
                .perform_series_form_sync_lazily(key, secret, unit_id, actual_download_id)
                .await
            {
                warn!("Background series form fetch failed: {}", e);
            } else {
                info!(
                    "[sync] Background series form fetch completed for series {}.",
                    unit_id
                );
                ctx_clone.refresh_from_db();
            }
        });
    }

    /// Performs the sync flow.
    ///
    /// 1. Tries to sync with stored secrets.
    /// 2. If that fails due to auth, starts the OAuth flow (Open Browser -> Get Code -> Verify -> Store).
    /// 3. Retries sync.
    /// 4. Reports progress via the provided sender.
    pub async fn perform_sync(
        window_weak: glib::WeakRef<NutmegWindow>,
        context: ContextObject,
        sender: tokio::sync::mpsc::UnboundedSender<(f64, String)>,
    ) {
        let db = Arc::new(DbManager::new());
        let sync = SyncService::new(db.clone());
        let key = crate::config::consumer_key();
        let secret = crate::config::consumer_secret();

        // Progress callback adapter
        let sender_clone = sender.clone();
        let progress_cb = Box::new(move |p: f64, msg: &str| {
            let _ = sender_clone.send((p, msg.to_string()));
        });

        let mut initial_fail_msg = None;

        match sync
            .perform_sync_with_stored_secrets(key.clone(), secret.clone(), progress_cb.clone())
            .await
        {
            Ok(Some((team_id, download_id))) => {
                info!("Sync completed successfully");
                context.refresh_from_db();
                Self::spawn_avatar_refresh(
                    db.clone(),
                    context.clone(),
                    key.clone(),
                    secret.clone(),
                    team_id,
                    download_id,
                );
            }
            Ok(None) => {
                warn!("Sync failed: No credentials found, starting OAuth flow...");
                // OAuth Flow
                if let Err(e) = Self::start_oauth_flow(
                    window_weak,
                    context,
                    &key,
                    &secret,
                    &sync,
                    db.clone(),
                    progress_cb,
                )
                .await
                {
                    error!("OAuth flow failed: {}", e);
                    initial_fail_msg = Some(format!("Auth failed: {}", e));
                }
            }
            Err(e) => {
                error!("Sync failed: {}", e);
                initial_fail_msg = Some(format!("Sync Error: {}", e));
            }
        }

        if let Some(msg) = initial_fail_msg {
            let _ = sender.send((0.0, msg));
        }
    }

    async fn start_oauth_flow(
        window_weak: glib::WeakRef<NutmegWindow>,
        context: ContextObject,
        key: &str,
        secret: &str,
        sync: &SyncService,
        db: Arc<DbManager>,
        progress_cb: ProgressCallback,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let auth_service = HattrickAuthService::new();
        let secret_service = SystemSecretService::new();

        // 1. Get Auth URL
        let (url, rt, rs) =
            tokio::task::spawn_blocking(move || auth_service.get_authorization_url()).await??;

        // 2. Open Browser
        open::that(&url)?;

        // 3. Show Dialog — must run on the GTK main thread
        let (tx, rx) = tokio::sync::oneshot::channel();

        let window_weak_clone = window_weak.clone();
        glib::MainContext::default().spawn_local(async move {
            if let Some(win) = window_weak_clone.upgrade() {
                let dialog = OAuthDialog::new(&win);
                let result = dialog.run().await;
                let _ = tx.send(result);
            } else {
                let _ = tx.send(None);
            }
        });

        let code_opt = rx.await.ok().flatten();

        if let Some(code) = code_opt {
            // 4. Verify Code
            let (token, token_secret) = tokio::task::spawn_blocking(move || {
                let auth_service = HattrickAuthService::new();
                auth_service.verify_user(&code, &rt, &rs)
            })
            .await??;

            // 5. Store Secrets
            secret_service.store_secret("access_token", &token).await?;
            secret_service
                .store_secret("access_secret", &token_secret)
                .await?;

            // 6. Retry Sync — pass the freshly obtained tokens directly instead of
            //    re-reading them from the keychain.
            match sync
                .perform_initial_sync(
                    key.to_string(),
                    secret.to_string(),
                    token,
                    token_secret,
                    progress_cb,
                )
                .await
            {
                Ok((team_id, download_id)) => {
                    info!("Retry sync successful");
                    context.refresh_from_db();
                    Self::spawn_avatar_refresh(
                        db,
                        context.clone(),
                        key.to_string(),
                        secret.to_string(),
                        team_id,
                        download_id,
                    );
                }
                Err(e) => return Err(format!("Retry sync error: {}", e).into()),
            }
        } else {
            warn!("User cancelled OAuth dialog");
            return Err("Cancelled".into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_controller_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<SyncController>();
    }
}
