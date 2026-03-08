use crate::chpp::client::HattrickClient;
use crate::config::{consumer_key, consumer_secret};
use crate::opponent_analysis::ui::model;
use crate::service::opponent_analysis::OpponentAnalysisService;
use crate::service::secret::SystemSecretService;
use crate::ui::context_object::ContextObject;
use crate::ui::team_object::TeamObject;
use gtk::gio;
use gtk::glib;
use gtk::prelude::*;
use log::debug;
use std::sync::Arc;

pub struct OpponentTabController {
    context: ContextObject,
}

impl OpponentTabController {
    pub fn new(context: ContextObject) -> Self {
        Self { context }
    }

    pub fn on_team_selected(&self, team: &TeamObject) {
        let our_team_id = team.team_data().id;
        let ctx = self.context.clone();

        let list_store = gio::ListStore::new::<model::OpponentItem>();
        ctx.set_upcoming_opponents(Some(list_store.clone()));

        let client = Arc::new(HattrickClient::new());
        let service = OpponentAnalysisService::new(client);

        // 1. Populate from DB immediately
        if let Ok(opponents) = service.get_upcoming_opponents_from_db(our_team_id) {
            for opp in opponents {
                let logo_url = format!(
                    "//res.hattrick.org/teamlogo/{}/{}/{}/{}/{}.png",
                    opp.team_id % 10,
                    opp.team_id % 100,
                    opp.team_id % 1000,
                    opp.team_id,
                    opp.team_id
                );
                let item = model::OpponentItem::new(
                    opp.team_id,
                    &opp.team_name,
                    &opp.match_date,
                    &logo_url,
                );
                list_store.append(&item);
            }
        }

        // 2. Refresh from API in background if secrets available
        glib::MainContext::default().spawn_local(async move {
            let secret_service = SystemSecretService::new();
            use crate::service::secret::SecretStorageService;

            let token_res = secret_service.get_secret("access_token").await;
            let secret_res = secret_service.get_secret("access_secret").await;

            if let (Ok(Some(token)), Ok(Some(secret))) = (token_res, secret_res) {
                let ck = consumer_key();
                let cs = consumer_secret();

                let get_auth =
                    || crate::chpp::oauth::create_oauth_context(&ck, &cs, &token, &secret);

                match service.get_upcoming_opponents(&get_auth, our_team_id).await {
                    Ok(opponents) => {
                        list_store.remove_all();
                        for opp in opponents {
                            let logo_url = format!(
                                "//res.hattrick.org/teamlogo/{}/{}/{}/{}/{}.png",
                                opp.team_id % 10,
                                opp.team_id % 100,
                                opp.team_id % 1000,
                                opp.team_id,
                                opp.team_id
                            );
                            let item = model::OpponentItem::new(
                                opp.team_id,
                                &opp.team_name,
                                &opp.match_date,
                                &logo_url,
                            );
                            list_store.append(&item);
                        }
                    }
                    Err(_e) => {
                        debug!("Failed to refresh upcoming opponents from API: {}", _e);
                    }
                }
            }
        });
    }

    pub fn clear(&self) {
        let store = gio::ListStore::new::<model::OpponentItem>();
        self.context.set_upcoming_opponents(Some(store));
    }
}
