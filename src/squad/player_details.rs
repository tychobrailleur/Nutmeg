use crate::ui::player_object::PlayerObject;
use gettextrs::gettext;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gdk, gio, glib, CompositeTemplate};
use log::debug;

// Shows the details of a specific player in the squad view.

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/Nutmeg/squad/player_details.ui")]
    pub struct SquadPlayerDetails {
        #[template_child]
        pub details_name: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_id: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_flag: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_gender: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_avatar: TemplateChild<gtk::Image>,

        // Category
        #[template_child]
        pub details_category: TemplateChild<gtk::Label>,

        // Level
        #[template_child]
        pub details_form: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_stamina: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_tsi: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_injury: TemplateChild<gtk::Label>,
        #[template_child]
        pub label_injury_title: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_specialty: TemplateChild<gtk::Label>,

        // Skills
        #[template_child]
        pub details_skill_keeper: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_skill_defender: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_skill_playmaker: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_skill_winger: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_skill_passing: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_skill_scorer: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_skill_set_pieces: TemplateChild<gtk::Label>,

        // Career / Club
        #[template_child]
        pub details_career_goals: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_league_goals: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_loyalty: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_mother_club: TemplateChild<gtk::LinkButton>,

        // Last Match
        #[template_child]
        pub details_last_match_date: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_played_minutes: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_position_code: TemplateChild<gtk::Label>,
        #[template_child]
        pub details_rating: TemplateChild<gtk::Label>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SquadPlayerDetails {
        const NAME: &'static str = "SquadPlayerDetails";
        type Type = super::SquadPlayerDetails;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SquadPlayerDetails {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }
    impl WidgetImpl for SquadPlayerDetails {}
    impl BoxImpl for SquadPlayerDetails {}
}

glib::wrapper! {
    pub struct SquadPlayerDetails(ObjectSubclass<imp::SquadPlayerDetails>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl SquadPlayerDetails {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn set_player(&self, player_obj: Option<PlayerObject>) {
        if let Some(player_obj) = player_obj {
            let imp = self.imp();
            let p = player_obj.player();
            self.set_visible(true);
            imp.details_name
                .set_label(&format!("{} {}", p.FirstName, p.LastName));
            imp.details_id.set_label(&p.PlayerID.to_string());

            // Flag: Prefer NativeCountryFlag, fallback to Flag (current country)
            let flag = p.NativeCountryFlag.as_ref().or(p.Flag.as_ref());
            if let Some(f) = flag {
                imp.details_flag.set_label(f);
                imp.details_flag.set_visible(true);
            } else {
                imp.details_flag.set_visible(false);
            }

            // Gender
            let gender_emoji = match p.GenderID {
                Some(2) => "♀️", // Female
                _ => "♂️",       // Male (default)
            };
            imp.details_gender.set_label(gender_emoji);

            // Avatar
            if let Some(blob) = &p.AvatarBlob {
                debug!(
                    "Player {} has avatar blob of size {}",
                    p.PlayerID,
                    blob.len()
                );
                let bytes = glib::Bytes::from(blob);
                let stream = gio::MemoryInputStream::from_bytes(&bytes);
                if let Ok(pixbuf) = gdk_pixbuf::Pixbuf::from_stream(&stream, gio::Cancellable::NONE)
                {
                    let texture = gdk::Texture::for_pixbuf(&pixbuf);
                    imp.details_avatar.set_paintable(Some(&texture));
                } else {
                    imp.details_avatar
                        .set_icon_name(Some("avatar-default-symbolic"));
                }
            } else {
                imp.details_avatar
                    .set_icon_name(Some("avatar-default-symbolic"));
            }

            // Category
            let cat_str = match p.PlayerCategoryId {
                Some(1) => gettext("Keeper"),
                Some(2) => gettext("Right Back"),
                Some(3) => gettext("Central Defender"),
                Some(4) => gettext("Winger"),
                Some(5) => gettext("Inner Midfielder"),
                Some(6) => gettext("Forward"),
                _ => gettext("Unknown/Unset"),
            };
            imp.details_category.set_label(&cat_str);

            // Level
            imp.details_form.set_label(&p.PlayerForm.to_string());

            let stamina = p
                .PlayerSkills
                .as_ref()
                .map(|s| s.StaminaSkill.to_string())
                .unwrap_or_else(|| "-".to_string());
            imp.details_stamina.set_label(&stamina);

            // TSI
            let locale = num_format::SystemLocale::default()
                .unwrap_or_else(|_| num_format::SystemLocale::from_name("C").unwrap());
            let mut buf_tsi = num_format::Buffer::default();
            buf_tsi.write_formatted(&p.TSI, &locale);
            imp.details_tsi.set_label(buf_tsi.as_str());

            // Injury
            if let Some(injury_level) = p.InjuryLevel {
                if injury_level == -1 {
                    imp.details_injury.set_visible(false);
                    imp.label_injury_title.set_visible(false);
                } else {
                    imp.details_injury.set_visible(true);
                    imp.label_injury_title.set_visible(true);
                    let injury_str = if injury_level == 0 {
                        "🩹".to_string()
                    } else {
                        format!("🚑 {} w", injury_level)
                    };
                    imp.details_injury.set_label(&injury_str);
                }
            } else {
                imp.details_injury.set_visible(false);
                imp.label_injury_title.set_visible(false);
            }

            // Specialty
            let specialty_str = match p.Specialty {
                Some(0) => gettext("No specialty"),
                Some(1) => gettext("Technical"),
                Some(2) => gettext("Quick"),
                Some(3) => gettext("Powerful"),
                Some(4) => gettext("Unpredictable"),
                Some(5) => gettext("Head specialist"),
                Some(6) => gettext("Resilient"),
                Some(8) => gettext("Support"),
                _ => "".to_string(),
            };
            imp.details_specialty.set_label(&specialty_str);

            // Skills
            let skills = p.PlayerSkills.as_ref();
            imp.details_skill_keeper.set_label(
                &skills
                    .map(|s| s.KeeperSkill.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            );
            imp.details_skill_defender.set_label(
                &skills
                    .map(|s| s.DefenderSkill.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            );
            imp.details_skill_playmaker.set_label(
                &skills
                    .map(|s| s.PlaymakerSkill.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            );
            imp.details_skill_winger.set_label(
                &skills
                    .map(|s| s.WingerSkill.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            );
            imp.details_skill_passing.set_label(
                &skills
                    .map(|s| s.PassingSkill.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            );
            imp.details_skill_scorer.set_label(
                &skills
                    .map(|s| s.ScorerSkill.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            );
            imp.details_skill_set_pieces.set_label(
                &skills
                    .map(|s| s.SetPiecesSkill.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            );

            // Career / Club
            imp.details_career_goals.set_label(
                &p.CareerGoals
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            );
            imp.details_league_goals.set_label(
                &p.LeagueGoals
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            );
            imp.details_loyalty.set_label(&p.Loyalty.to_string());

            // Mother Club
            debug!(
                "MotherClub: {:?}, Bonus: {}",
                p.MotherClub, p.MotherClubBonus
            );
            if let Some(mother_club) = &p.MotherClub {
                if !p.MotherClubBonus {
                    imp.details_mother_club.set_label(&mother_club.TeamName);
                    imp.details_mother_club.set_uri(&format!(
                        "https://www.hattrick.org/Club/?TeamID={}",
                        mother_club.TeamID
                    ));
                    imp.details_mother_club.set_visible(true);
                } else {
                    imp.details_mother_club.set_visible(false);
                }
            } else if p.MotherClubBonus {
                imp.details_mother_club.set_label(&gettext("Home Grown"));
                imp.details_mother_club.set_sensitive(false); // No link
                imp.details_mother_club.set_visible(true);
            } else {
                imp.details_mother_club.set_visible(false);
            }

            // Last Match
            imp.details_last_match_date
                .set_label(p.LastMatch.as_ref().map(|m| m.Date.as_str()).unwrap_or("-"));
            imp.details_played_minutes.set_label(
                &p.LastMatch
                    .as_ref()
                    .map(|m| m.PlayedMinutes.to_string())
                    .unwrap_or_else(|| "-".to_string()),
            );
            imp.details_position_code.set_label(
                &p.LastMatch
                    .as_ref()
                    .map(|m| crate::ui::player_display::translate_position_id(m.PositionCode))
                    .unwrap_or_else(|| "-".to_string()),
            );

            let rating_str = p
                .LastMatch
                .as_ref()
                .and_then(|m| m.Rating)
                .map(|r| r.to_string())
                .unwrap_or_else(|| "-".to_string());
            imp.details_rating.set_label(&rating_str);
        } else {
            self.set_visible(false);
        }
    }
}

impl Default for SquadPlayerDetails {
    fn default() -> Self {
        Self::new()
    }
}
