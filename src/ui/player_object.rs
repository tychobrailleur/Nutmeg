use crate::chpp::model::Player;
use gtk::glib;
use gtk::subclass::prelude::*;
use std::cell::RefCell;

// Wraps a Player in a GObject for use in the UI.

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct PlayerObject {
        pub data: RefCell<Option<Player>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for PlayerObject {
        const NAME: &'static str = "PlayerObject";
        type Type = super::PlayerObject;
    }

    impl ObjectImpl for PlayerObject {}
}

glib::wrapper! {
    pub struct PlayerObject(ObjectSubclass<imp::PlayerObject>);
}

impl PlayerObject {
    pub fn new(player: Player) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp().data.replace(Some(player));
        obj
    }

    pub fn player(&self) -> Player {
        self.imp().data.borrow().as_ref().unwrap().clone()
    }
}
