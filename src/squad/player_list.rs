use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/Nutmeg/squad/player_list.ui")]
    pub struct SquadPlayerList {
        #[template_child]
        pub view_players: TemplateChild<gtk::TreeView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SquadPlayerList {
        const NAME: &'static str = "SquadPlayerList";
        type Type = super::SquadPlayerList;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SquadPlayerList {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_tree_view();
        }
    }
    impl WidgetImpl for SquadPlayerList {}
    impl BoxImpl for SquadPlayerList {}
}

glib::wrapper! {
    pub struct SquadPlayerList(ObjectSubclass<imp::SquadPlayerList>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl SquadPlayerList {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn tree_view(&self) -> gtk::TreeView {
        self.imp().view_players.clone()
    }

    fn setup_tree_view(&self) {
        let imp = self.imp();
        let view = &imp.view_players;

        // Helper to add a text column
        let add_column = |title: &str, col_id: i32| {
            let renderer = gtk::CellRendererText::new();
            let column = gtk::TreeViewColumn::new();
            column.set_title(title);
            column.set_reorderable(true);
            column.set_resizable(true);
            column.pack_start(&renderer, true);
            column.add_attribute(&renderer, "text", col_id);
            column.add_attribute(&renderer, "cell-background", 13); // BG Color is now at index 13
            view.append_column(&column);
        };

        // Columns:
        // 0: Name, 1: Flag, 2: Number, 3: Age, 4: Form, 5: TSI
        // 6: Salary, 7: Specialty, 8: Experience, 9: Leadership, 10: Loyalty
        // 11: Best Pos, 12: Last Pos, 13: BG Color, 14: Stamina, 15: Injured, 16: Cards, 17: Mother Club
        // 18: PlayerObj

        add_column(&gettext("Name"), 0);
        add_column(&gettext("Flag"), 1);
        add_column(&gettext("No."), 2);
        add_column(&gettext("Age"), 3);
        add_column(&gettext("Form"), 4);
        add_column(&gettext("TSI"), 5);
        add_column(&gettext("Salary"), 6);
        add_column(&gettext("Specialty"), 7);
        add_column(&gettext("XP"), 8);
        add_column(&gettext("Lead"), 9);
        add_column(&gettext("Loyalty"), 10);
        add_column(&gettext("Best Pos"), 11);
        add_column(&gettext("Last Pos"), 12);
        // BG Color is 13, not displayed as column
        add_column(&gettext("Stamina"), 14);
        add_column(&gettext("Injured"), 15);
        add_column(&gettext("Cards"), 16);
        add_column(&gettext("Mother Club"), 17);
    }
}

use crate::ui::player_display::PlayerDisplay;
use crate::ui::player_object::PlayerObject;
use num_format::SystemLocale;

pub fn create_player_model(players: &[crate::chpp::model::Player]) -> gtk::ListStore {
    #[allow(deprecated)]
    let store = gtk::ListStore::new(&[
        glib::Type::STRING, // 0 Name
        glib::Type::STRING, // 1 Flag
        glib::Type::STRING, // 2 Number
        glib::Type::STRING, // 3 Age
        glib::Type::STRING, // 4 Form
        glib::Type::STRING, // 5 TSI
        glib::Type::STRING, // 6 Salary
        glib::Type::STRING, // 7 Specialty
        glib::Type::STRING, // 8 Experience
        glib::Type::STRING, // 9 Leadership
        glib::Type::STRING, // 10 Loyalty
        glib::Type::STRING, // 11 Best Position
        glib::Type::STRING, // 12 Last Position
        glib::Type::STRING, // 13 BG Color
        glib::Type::STRING, // 14 Stamina
        glib::Type::STRING, // 15 Injured
        glib::Type::STRING, // 16 Cards
        glib::Type::STRING, // 17 Mother Club
        glib::Type::OBJECT, // 18 PlayerObject
    ]);

    let locale = SystemLocale::default().unwrap_or_else(|_| SystemLocale::from_name("C").unwrap());

    for p in players {
        let obj = PlayerObject::new(p.clone());
        let display = PlayerDisplay::new(&p, &locale);

        let bg = if p.MotherClubBonus {
            Some("rgba(64, 224, 208, 0.3)")
        } else {
            None
        };

        store.insert_with_values(
            None,
            &[
                (0, &display.name),
                (1, &display.flag),
                (2, &display.number),
                (3, &display.age),
                (4, &display.form),
                (5, &display.tsi),
                (6, &display.salary),
                (7, &display.specialty),
                (8, &display.xp),
                (9, &display.leadership),
                (10, &display.loyalty),
                (11, &display.best_pos),
                (12, &display.last_pos),
                (13, &bg),
                (14, &display.stamina),
                (15, &display.injured),
                (16, &display.cards),
                (17, &display.mother_club),
                (18, &obj),
            ],
        );
    }
    store
}
