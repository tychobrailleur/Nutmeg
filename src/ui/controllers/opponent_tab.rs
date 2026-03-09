use crate::chpp::client::HattrickClient;
use crate::opponent_analysis::ui::model;
use crate::service::opponent_analysis::OpponentAnalysisService;
use crate::ui::context_object::ContextObject;
use crate::ui::team_object::TeamObject;
use gtk::gio;
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
    }

    pub fn clear(&self) {
        let store = gio::ListStore::new::<model::OpponentItem>();
        self.context.set_upcoming_opponents(Some(store));
    }
}
