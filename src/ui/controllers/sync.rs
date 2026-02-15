use gtk::prelude::*;
use gtk::{glib, gio};
use log::{debug, error, info, warn};
use std::sync::Arc;
use crate::service::sync::{DataSyncService, SyncService};
use crate::db::manager::DbManager;
use crate::ui::oauth_dialog::OAuthDialog;
use crate::service::auth::{AuthenticationService, HattrickAuthService};
use crate::service::secret::{GnomeSecretService, SecretStorageService};
use crate::window::NutmegWindow;

pub struct SyncController;

impl SyncController {
    /// Performs the sync flow.
    /// 
    /// 1. Tries to sync with stored secrets.
    /// 2. If that fails due to auth, starts the OAuth flow (Open Browser -> Get Code -> Verify -> Store).
    /// 3. Retries sync.
    /// 4. Reports progress via the provided sender.
    pub async fn perform_sync(
        window_weak: glib::WeakRef<NutmegWindow>,
        sender: tokio::sync::mpsc::UnboundedSender<(f64, String)>
    ) {
        let db = Arc::new(DbManager::new());
        let sync = SyncService::new(db);
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
            Ok(true) => {
                info!("Sync completed successfully");
            }
            Ok(false) => {
                warn!("Sync failed: No credentials found, starting OAuth flow...");
                // OAuth Flow
                if let Err(e) = Self::start_oauth_flow(window_weak, &key, &secret, &sync, progress_cb).await {
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
        key: &str,
        secret: &str,
        sync: &SyncService,
        progress_cb: Box<dyn Fn(f64, &str) + Send + Sync>
    ) -> Result<(), Box<dyn std::error::Error>> {
        let auth_service = HattrickAuthService::new();
        let secret_service = GnomeSecretService::new();

        // 1. Get Auth URL
        let (url, rt, rs) = tokio::task::spawn_blocking(move || {
                auth_service.get_authorization_url()
        }).await??;

        // 2. Open Browser
        open::that(&url)?;

        // 3. Show Dialog (UI Thread)
        // We need to switch to MainContext to show the dialog
        let (tx, rx) = tokio::sync::oneshot::channel();
        
        let window_weak_clone = window_weak.clone();
        glib::MainContext::default().spawn_local(async move {
            if let Some(win) = window_weak_clone.upgrade() {
                // OAuthDialog expects &impl IsA<gtk::Window>
                // NutmegWindow implements IsA<gtk::Window>
                let dialog = OAuthDialog::new(&win);
                let result = dialog.run().await;
                let _ = tx.send(result);
            } else {
                let _ = tx.send(None);
            }
        });

        let code_opt = rx.await.unwrap_or(None);

        if let Some(code) = code_opt {
             // 4. Verify Code
            let (token, token_secret) = tokio::task::spawn_blocking(move || {
                    let auth_service = HattrickAuthService::new();
                    auth_service.verify_user(&code, &rt, &rs)
            }).await??;

            // 5. Store Secrets
            secret_service.store_secret("access_token", &token).await?;
            secret_service.store_secret("access_secret", &token_secret).await?;

            // 6. Retry Sync
            match sync.perform_sync_with_stored_secrets(key.to_string(), secret.to_string(), progress_cb).await {
                Ok(true) => info!("Retry sync successful"),
                Ok(false) => return Err("Retry sync failed (still no creds?)".into()),
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
