use crate::db::manager::DbManager;
use crate::ui::context_object::ContextObject;
use crate::ui::controllers::opponent_tab::OpponentTabController;
use crate::ui::controllers::series_tab::SeriesTabController;
use crate::ui::controllers::squad_tab::SquadTabController;
use crate::ui::team_object::TeamObject;
use gtk::prelude::*;
use std::rc::Rc;

pub struct MainController {
    context: ContextObject,
    series_tab: SeriesTabController,
    squad_tab: SquadTabController,
    opponent_tab: OpponentTabController,
}

impl MainController {
    pub fn new(context: ContextObject) -> Rc<Self> {
        let series_tab = SeriesTabController::new(context.clone());
        let squad_tab = SquadTabController::new(context.clone());
        let opponent_tab = OpponentTabController::new(context.clone());

        let controller = Rc::new(Self {
            context,
            series_tab,
            squad_tab,
            opponent_tab,
        });
        controller.setup_bindings();
        controller
    }

    pub fn context(&self) -> &ContextObject {
        &self.context
    }

    fn setup_bindings(self: &Rc<Self>) {
        let weak_self = Rc::downgrade(self);

        // Listen for team selection changes
        self.context
            .connect_notify_local(Some("selected-team"), move |ctx, _| {
                if let Some(ctrl) = weak_self.upgrade() {
                    if let Some(team) = ctx.property::<Option<TeamObject>>("selected-team") {
                        ctrl.load_team_data(team);
                    } else {
                        ctrl.clear_team_data();
                    }
                }
            });
    }

    pub fn refresh_all_teams(&self) {
        let db = DbManager::new();
        if let Ok(mut conn) = db.get_connection() {
            if let Ok(teams) = crate::db::teams::get_teams_summary(&mut conn) {
                let model = gtk::gio::ListStore::new::<TeamObject>();
                for (id, name, logo_url) in teams {
                    model.append(&TeamObject::new(id, name, logo_url));
                }
                self.context.set_all_teams(Some(model));
            }
        }
    }

    fn load_team_data(&self, _team: TeamObject) {
        // No-op: ContextObject now refreshes itself on selected-team change.
    }

    fn clear_team_data(&self) {
        // No-op: ContextObject now clears itself via refresh_from_db() -> clear_context().
    }
}
