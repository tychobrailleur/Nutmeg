use crate::db::manager::DbManager;
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

    pub fn clear(&self) {
        // No-op: ContextObject clears its own properties.
    }
}
