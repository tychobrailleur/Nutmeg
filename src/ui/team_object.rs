use gtk::glib;
use gtk::subclass::prelude::*;
use std::cell::RefCell;

#[derive(Clone, Debug)]
pub struct TeamData {
    pub id: u32,
    pub name: String,
    pub logo_url: Option<String>,
}

mod imp {
    use super::*;

    #[derive(Default)]
    pub struct TeamObject {
        pub data: RefCell<Option<TeamData>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TeamObject {
        const NAME: &'static str = "TeamObject";
        type Type = super::TeamObject;
    }

    impl ObjectImpl for TeamObject {}
}

glib::wrapper! {
    pub struct TeamObject(ObjectSubclass<imp::TeamObject>);
}

impl TeamObject {
    pub fn new(id: u32, name: String, logo_url: Option<String>) -> Self {
        let obj: Self = glib::Object::new();
        obj.imp()
            .data
            .replace(Some(TeamData { id, name, logo_url }));
        obj
    }

    pub fn team_data(&self) -> TeamData {
        self.imp().data.borrow().as_ref().unwrap().clone()
    }
}
