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
        pub home_team_name: RefCell<String>,
        pub away_team_name: RefCell<String>,
        pub opponent_team_name: RefCell<String>,
        pub match_type: RefCell<u32>,
        pub home_goals: RefCell<Option<u32>>,
        pub away_goals: RefCell<Option<u32>>,
        pub formation: RefCell<Option<String>>,
        pub tactic_type: RefCell<Option<u32>>,
        pub rating_midfield: RefCell<Option<u32>>,
        pub rating_right_def: RefCell<Option<u32>>,
        pub rating_mid_def: RefCell<Option<u32>>,
        pub rating_left_def: RefCell<Option<u32>>,
        pub rating_right_att: RefCell<Option<u32>>,
        pub rating_mid_att: RefCell<Option<u32>>,
        pub rating_left_att: RefCell<Option<u32>>,
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
                    glib::ParamSpecString::builder("home-team-name").build(),
                    glib::ParamSpecString::builder("away-team-name").build(),
                    glib::ParamSpecString::builder("opponent-team-name").build(),
                    glib::ParamSpecUInt::builder("match-type").build(),
                ]
            })
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "match-id" => *self.match_id.borrow_mut() = value.get().unwrap(),
                "match-date" => *self.match_date.borrow_mut() = value.get().unwrap(),
                "is-home" => *self.is_home.borrow_mut() = value.get().unwrap(),
                "home-team-name" => *self.home_team_name.borrow_mut() = value.get().unwrap(),
                "away-team-name" => *self.away_team_name.borrow_mut() = value.get().unwrap(),
                "opponent-team-name" => {
                    *self.opponent_team_name.borrow_mut() = value.get().unwrap()
                }
                "match-type" => *self.match_type.borrow_mut() = value.get().unwrap(),
                _ => unimplemented!(),
            }
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "match-id" => self.match_id.borrow().to_value(),
                "match-date" => self.match_date.borrow().to_value(),
                "is-home" => self.is_home.borrow().to_value(),
                "home-team-name" => self.home_team_name.borrow().to_value(),
                "away-team-name" => self.away_team_name.borrow().to_value(),
                "opponent-team-name" => self.opponent_team_name.borrow().to_value(),
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
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        match_id: u32,
        match_date: &str,
        is_home: bool,
        home_team_name: &str,
        away_team_name: &str,
        opponent_team_name: &str,
        match_type: u32,
        home_goals: Option<u32>,
        away_goals: Option<u32>,
        formation: Option<String>,
        tactic_type: Option<u32>,
        rating_midfield: Option<u32>,
        rating_right_def: Option<u32>,
        rating_mid_def: Option<u32>,
        rating_left_def: Option<u32>,
        rating_right_att: Option<u32>,
        rating_mid_att: Option<u32>,
        rating_left_att: Option<u32>,
    ) -> Self {
        let obj = glib::Object::builder::<Self>()
            .property("match-id", match_id)
            .property("match-date", match_date)
            .property("is-home", is_home)
            .property("home-team-name", home_team_name)
            .property("away-team-name", away_team_name)
            .property("opponent-team-name", opponent_team_name)
            .property("match-type", match_type)
            .build();

        // Because Option<T> is tricky with simple properties, we simply mutate the ref cell directly
        *obj.imp().home_goals.borrow_mut() = home_goals;
        *obj.imp().away_goals.borrow_mut() = away_goals;
        *obj.imp().formation.borrow_mut() = formation;
        *obj.imp().tactic_type.borrow_mut() = tactic_type;
        *obj.imp().rating_midfield.borrow_mut() = rating_midfield;
        *obj.imp().rating_right_def.borrow_mut() = rating_right_def;
        *obj.imp().rating_mid_def.borrow_mut() = rating_mid_def;
        *obj.imp().rating_left_def.borrow_mut() = rating_left_def;
        *obj.imp().rating_right_att.borrow_mut() = rating_right_att;
        *obj.imp().rating_mid_att.borrow_mut() = rating_mid_att;
        *obj.imp().rating_left_att.borrow_mut() = rating_left_att;

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

    pub fn home_team_name(&self) -> String {
        self.property("home-team-name")
    }

    pub fn away_team_name(&self) -> String {
        self.property("away-team-name")
    }

    pub fn opponent_team_name(&self) -> String {
        self.property("opponent-team-name")
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

    pub fn tactic_type(&self) -> Option<u32> {
        *self.imp().tactic_type.borrow()
    }

    pub fn rating_midfield(&self) -> Option<u32> {
        *self.imp().rating_midfield.borrow()
    }

    pub fn rating_right_def(&self) -> Option<u32> {
        *self.imp().rating_right_def.borrow()
    }

    pub fn rating_mid_def(&self) -> Option<u32> {
        *self.imp().rating_mid_def.borrow()
    }

    pub fn rating_left_def(&self) -> Option<u32> {
        *self.imp().rating_left_def.borrow()
    }

    pub fn rating_right_att(&self) -> Option<u32> {
        *self.imp().rating_right_att.borrow()
    }

    pub fn rating_mid_att(&self) -> Option<u32> {
        *self.imp().rating_mid_att.borrow()
    }

    pub fn rating_left_att(&self) -> Option<u32> {
        *self.imp().rating_left_att.borrow()
    }
}
