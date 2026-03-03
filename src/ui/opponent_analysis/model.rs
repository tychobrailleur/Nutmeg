use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    pub struct OpponentItem {
        pub team_id: RefCell<u32>,
        pub team_name: RefCell<String>,
        pub match_date: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for OpponentItem {
        const NAME: &'static str = "OpponentItem";
        type Type = super::OpponentItem;
    }

    impl ObjectImpl for OpponentItem {
        fn properties() -> &'static [glib::ParamSpec] {
            use std::sync::OnceLock;
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecUInt::builder("team-id").build(),
                    glib::ParamSpecString::builder("team-name").build(),
                    glib::ParamSpecString::builder("match-date").build(),
                    glib::ParamSpecString::builder("display-text")
                        .read_only()
                        .build(),
                ]
            })
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "team-id" => {
                    *self.team_id.borrow_mut() = value.get().unwrap();
                }
                "team-name" => {
                    *self.team_name.borrow_mut() = value.get().unwrap();
                }
                "match-date" => {
                    *self.match_date.borrow_mut() = value.get().unwrap();
                }
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "team-id" => self.team_id.borrow().to_value(),
                "team-name" => self.team_name.borrow().to_value(),
                "match-date" => self.match_date.borrow().to_value(),
                "display-text" => {
                    let text =
                        format!("{} ({})", self.team_name.borrow(), self.match_date.borrow());
                    text.to_value()
                }
                _ => unimplemented!(),
            }
        }
    }

    #[derive(Default)]
    pub struct MatchItem {
        pub match_id: RefCell<u32>,
        pub match_date: RefCell<String>,
        pub is_home: RefCell<bool>,
        pub match_type: RefCell<u32>,
        pub home_goals: RefCell<Option<u32>>,
        pub away_goals: RefCell<Option<u32>>,
        pub formation: RefCell<Option<String>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MatchItem {
        const NAME: &'static str = "MatchItem";
        type Type = super::MatchItem;
    }

    impl ObjectImpl for MatchItem {
        fn properties() -> &'static [glib::ParamSpec] {
            use std::sync::OnceLock;
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecUInt::builder("match-id").build(),
                    glib::ParamSpecString::builder("match-date").build(),
                    glib::ParamSpecBoolean::builder("is-home").build(),
                    glib::ParamSpecUInt::builder("match-type").build(),
                ]
            })
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "match-id" => *self.match_id.borrow_mut() = value.get().unwrap(),
                "match-date" => *self.match_date.borrow_mut() = value.get().unwrap(),
                "is-home" => *self.is_home.borrow_mut() = value.get().unwrap(),
                "match-type" => *self.match_type.borrow_mut() = value.get().unwrap(),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "match-id" => self.match_id.borrow().to_value(),
                "match-date" => self.match_date.borrow().to_value(),
                "is-home" => self.is_home.borrow().to_value(),
                "match-type" => self.match_type.borrow().to_value(),
                _ => unimplemented!(),
            }
        }
    }
}

glib::wrapper! {
    pub struct OpponentItem(ObjectSubclass<imp::OpponentItem>);
}

glib::wrapper! {
    pub struct MatchItem(ObjectSubclass<imp::MatchItem>);
}

impl OpponentItem {
    pub fn new(team_id: u32, team_name: &str, match_date: &str) -> Self {
        glib::Object::builder()
            .property("team-id", team_id)
            .property("team-name", team_name)
            .property("match-date", match_date)
            .build()
    }

    pub fn team_id(&self) -> u32 {
        self.property("team-id")
    }
}

impl MatchItem {
    pub fn new(
        match_id: u32,
        match_date: &str,
        is_home: bool,
        match_type: u32,
        home_goals: Option<u32>,
        away_goals: Option<u32>,
        formation: Option<String>,
    ) -> Self {
        let obj = glib::Object::builder::<Self>()
            .property("match-id", match_id)
            .property("match-date", match_date)
            .property("is-home", is_home)
            .property("match-type", match_type)
            .build();

        // Because Option<T> is tricky with simple properties, we simply mutate the ref cell directly
        *obj.imp().home_goals.borrow_mut() = home_goals;
        *obj.imp().away_goals.borrow_mut() = away_goals;
        *obj.imp().formation.borrow_mut() = formation;

        obj
    }

    pub fn match_id(&self) -> u32 {
        self.property("match-id")
    }

    pub fn match_date(&self) -> String {
        self.property("match-date")
    }

    pub fn is_home(&self) -> bool {
        self.property("is-home")
    }

    pub fn match_type(&self) -> u32 {
        self.property("match-type")
    }

    pub fn home_goals(&self) -> Option<u32> {
        *self.imp().home_goals.borrow()
    }

    pub fn away_goals(&self) -> Option<u32> {
        *self.imp().away_goals.borrow()
    }

    pub fn formation(&self) -> Option<String> {
        self.imp().formation.borrow().clone()
    }
}
