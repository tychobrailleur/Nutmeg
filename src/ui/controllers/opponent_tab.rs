use crate::ui::context_object::ContextObject;

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
