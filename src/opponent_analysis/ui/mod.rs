/* mod.rs
 *
 * Copyright 2026 Sébastien Le Callonnec
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, Button, Label, SpinButton, Spinner};
use log::debug;
use std::sync::Arc;

use crate::chpp::client::HattrickClient;
use crate::config::{consumer_key, consumer_secret};
use crate::service::opponent_analysis::OpponentAnalysisService;
use crate::service::secret::SystemSecretService;
use gettextrs::gettext;

pub mod model;

mod imp {
    use super::*;
    use gtk::CompositeTemplate;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/Nutmeg/opponent_analysis/ui/opponent_analysis.ui")]
    pub struct OpponentAnalysis {
        #[template_child]
        pub dropdown_team: TemplateChild<gtk::DropDown>,
        #[template_child]
        pub spin_limit: TemplateChild<SpinButton>,
        #[template_child]
        pub btn_analyze: TemplateChild<Button>,
        #[template_child]
        pub spinner: TemplateChild<Spinner>,
        #[template_child]
        pub lbl_formations: TemplateChild<Label>,
        #[template_child]
        pub lbl_unavailable: TemplateChild<Label>,
        #[template_child]
        pub average_ratings_pitch: TemplateChild<gtk::Box>,
        #[template_child]
        pub match_list_view: TemplateChild<gtk::ListView>,
        #[template_child]
        pub pitch_container: TemplateChild<gtk::Box>,
        #[template_child]
        pub check_league: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub check_cup: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub check_friendly: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub check_international: TemplateChild<gtk::CheckButton>,
        #[template_child]
        pub lbl_tactical_summary: TemplateChild<Label>,

        pub selected_team: std::cell::RefCell<Option<crate::ui::team_object::TeamObject>>,
        pub selected_opponent: std::cell::RefCell<Option<model::OpponentItem>>,
        pub context: std::cell::RefCell<Option<crate::ui::context_object::ContextObject>>,
        pub latest_matches:
            std::cell::RefCell<Vec<crate::service::opponent_analysis::OpponentMatchData>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for OpponentAnalysis {
        const NAME: &'static str = "OpponentAnalysis";
        type Type = super::OpponentAnalysis;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for OpponentAnalysis {
        fn properties() -> &'static [glib::ParamSpec] {
            use std::sync::OnceLock;
            static PROPERTIES: OnceLock<Vec<glib::ParamSpec>> = OnceLock::new();
            PROPERTIES.get_or_init(|| {
                vec![
                    glib::ParamSpecObject::builder::<crate::ui::team_object::TeamObject>(
                        "selected-team",
                    )
                    .explicit_notify()
                    .build(),
                    glib::ParamSpecObject::builder::<crate::ui::context_object::ContextObject>(
                        "context",
                    )
                    .explicit_notify()
                    .build(),
                    glib::ParamSpecObject::builder::<model::OpponentItem>("selected-opponent")
                        .explicit_notify()
                        .build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "selected-team" => self.selected_team.borrow().to_value(),
                "context" => self.context.borrow().to_value(),
                "selected-opponent" => self.selected_opponent.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "selected-team" => {
                    let team = value
                        .get::<Option<crate::ui::team_object::TeamObject>>()
                        .expect("Value must be Option<TeamObject>");
                    self.selected_team.replace(team);
                    self.obj().notify("selected-team");
                }
                "context" => {
                    let ctx = value
                        .get::<Option<crate::ui::context_object::ContextObject>>()
                        .expect("Value must be Option<ContextObject>");
                    if let Some(c) = &ctx {
                        let dropdown: &gtk::DropDown = &self.dropdown_team;
                        c.bind_property("upcoming-opponents", dropdown, "model")
                            .flags(glib::BindingFlags::SYNC_CREATE)
                            .build();
                    }
                    self.context.replace(ctx);
                    self.obj().notify("context");
                }
                "selected-opponent" => {
                    let item = value
                        .get::<Option<model::OpponentItem>>()
                        .expect("Value must be Option<OpponentItem>");
                    self.selected_opponent.replace(item);
                    self.obj().notify("selected-opponent");
                }
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();

            // Setup the Team DropDown Factory
            let team_factory = gtk::SignalListItemFactory::new();
            team_factory.connect_setup(|_, list_item| {
                let list_item = list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Needs to be ListItem");
                let hbox = gtk::Box::builder()
                    .orientation(gtk::Orientation::Horizontal)
                    .spacing(8)
                    .margin_start(4)
                    .margin_end(4)
                    .margin_top(4)
                    .margin_bottom(4)
                    .build();

                let img = gtk::Image::builder()
                    .pixel_size(24)
                    .halign(gtk::Align::Center)
                    .valign(gtk::Align::Center)
                    .build();

                let lbl = gtk::Label::builder().halign(gtk::Align::Start).build();

                hbox.append(&img);
                hbox.append(&lbl);
                list_item.set_child(Some(&hbox));
            });

            team_factory.connect_bind(|_, list_item| {
                let list_item = list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Needs to be ListItem");
                let team_item = list_item
                    .item()
                    .and_downcast::<model::OpponentItem>()
                    .expect("Item is not an OpponentItem");
                let hbox = list_item
                    .child()
                    .and_downcast::<gtk::Box>()
                    .expect("Child is not a GtkBox");

                let img = hbox
                    .first_child()
                    .unwrap()
                    .downcast::<gtk::Image>()
                    .unwrap();
                let lbl = img
                    .next_sibling()
                    .unwrap()
                    .downcast::<gtk::Label>()
                    .unwrap();

                let img_clone = img.clone();
                let url = team_item.team_logo_url();

                // Set a placeholder while loading
                img.set_icon_name(Some("image-missing"));

                glib::MainContext::default().spawn_local(async move {
                    if let Ok(texture) = crate::utils::image::load_image_from_url(&url).await {
                        img_clone.set_paintable(Some(&texture));
                    }
                });
                lbl.set_text(&team_item.property::<String>("display-text"));
            });

            self.dropdown_team.set_factory(Some(&team_factory));

            // Setup the Match List Factory
            let factory = gtk::SignalListItemFactory::new();
            factory.connect_setup(|_, list_item| {
                let list_item = list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Needs to be ListItem");
                let vbox = gtk::Box::builder()
                    .orientation(gtk::Orientation::Vertical)
                    .spacing(4)
                    .margin_start(8)
                    .margin_end(8)
                    .margin_top(8)
                    .margin_bottom(8)
                    .build();

                let lbl_date = gtk::Label::builder()
                    .halign(gtk::Align::Start)
                    .css_classes(["strong"])
                    .build();
                let lbl_score = gtk::Label::builder().halign(gtk::Align::Start).build();
                let lbl_desc = gtk::Label::builder()
                    .halign(gtk::Align::Start)
                    .css_classes(["dim-label"])
                    .build();
                let lbl_ratings = gtk::Label::builder()
                    .halign(gtk::Align::Start)
                    .css_classes(["dim-label"])
                    .build();

                vbox.append(&lbl_date);
                vbox.append(&lbl_score);
                vbox.append(&lbl_desc);
                vbox.append(&lbl_ratings);

                list_item.set_child(Some(&vbox));
            });
            factory.connect_bind(|_, list_item| {
                let list_item = list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Needs to be ListItem");
                let match_item = list_item
                    .item()
                    .and_downcast::<model::MatchItem>()
                    .expect("Item is not a MatchItem");
                let vbox = list_item
                    .child()
                    .and_downcast::<gtk::Box>()
                    .expect("Child is not a GtkBox");

                let lbl_date = vbox
                    .first_child()
                    .unwrap()
                    .downcast::<gtk::Label>()
                    .unwrap();
                let lbl_score = lbl_date
                    .next_sibling()
                    .unwrap()
                    .downcast::<gtk::Label>()
                    .unwrap();
                let lbl_desc = lbl_score
                    .next_sibling()
                    .unwrap()
                    .downcast::<gtk::Label>()
                    .unwrap();
                let lbl_ratings = lbl_desc
                    .next_sibling()
                    .unwrap()
                    .downcast::<gtk::Label>()
                    .unwrap();

                // Format Score & Match Name
                let is_home = match_item.is_home();
                let hg = match_item.home_goals();
                let ag = match_item.away_goals();

                let score_str = format!(
                    "{} {} – {} {}", //FIXME: Make this localisable.
                    match_item.home_team_name(),
                    hg.unwrap_or(0),
                    ag.unwrap_or(0),
                    match_item.away_team_name()
                );

                // Format Tactics & Formation
                let form_str = match_item.formation().unwrap_or_else(|| gettext("Unknown"));
                let tactic_name = match match_item.tactic_type().unwrap_or(0) {
                    1 => gettext("Pressing"),
                    2 => gettext("Counter-attacks"),
                    3 => gettext("Attack in the middle"),
                    4 => gettext("Attack on wings"),
                    7 => gettext("Play creatively"),
                    8 => gettext("Long shots"),
                    _ => gettext("Normal"),
                };
                let match_type_str = match match_item.match_type() {
                    1 | 2 => gettext("League"),
                    3 | 7 | 8 => gettext("Cup"),
                    4 | 5 | 9 => gettext("Friendly"),
                    50 | 51 => gettext("Tournament"),
                    62 | 63 => gettext("Ladder"),
                    _ => gettext("Other"),
                };

                lbl_date.set_text(&match_item.match_date());
                lbl_score.set_text(&score_str);
                lbl_desc.set_text(&format!(
                    "{} | {} | {}",
                    match_type_str, tactic_name, form_str
                ));

                // Format Sector Ratings
                // Extracting as (Midfield, Defense, Attack) total approximations.
                let mid = match_item.rating_midfield().unwrap_or(0) as f32;
                let def = (match_item.rating_left_def().unwrap_or(0)
                    + match_item.rating_mid_def().unwrap_or(0)
                    + match_item.rating_right_def().unwrap_or(0)) as f32
                    / 3.0;
                let att = (match_item.rating_left_att().unwrap_or(0)
                    + match_item.rating_mid_att().unwrap_or(0)
                    + match_item.rating_right_att().unwrap_or(0)) as f32
                    / 3.0;

                lbl_ratings.set_text(&format!(
                    "{}: {:.1} | {}: {:.1} | {}: {:.1}",
                    gettext("Def"),
                    def,
                    gettext("Mid"),
                    mid,
                    gettext("Att"),
                    att
                ));
            });

            self.match_list_view.set_factory(Some(&factory));

            let store = gio::ListStore::new::<model::MatchItem>();
            let selection_model = gtk::SingleSelection::new(Some(store));
            self.match_list_view.set_model(Some(&selection_model));

            // Handle match selection
            let pitch_container = self.pitch_container.clone();
            let team_dropdown = self.dropdown_team.clone();
            selection_model.connect_selected_item_notify(move |sel| {
                if let Some(item) = sel.selected_item() {
                    if let Ok(match_item) = item.downcast::<model::MatchItem>() {
                        let match_id = match_item.match_id();

                        let selected_team = team_dropdown.selected_item();
                        let team_id =
                            if let Some(t) = selected_team.and_downcast::<model::OpponentItem>() {
                                t.team_id()
                            } else {
                                return;
                            };

                        let pitch_container_clone = pitch_container.clone();

                        glib::MainContext::default().spawn_local(async move {
                            // TODO Check whether this should be in a service
                            let secret_service = crate::service::secret::SystemSecretService::new();
                            use crate::service::secret::SecretStorageService;

                            let token_res = secret_service.get_secret("access_token").await;
                            let secret_res = secret_service.get_secret("access_secret").await;

                            if let (Ok(Some(token)), Ok(Some(secret))) = (token_res, secret_res) {
                                let client = Arc::new(crate::chpp::client::HattrickClient::new());
                                let service =
                                    crate::service::opponent_analysis::OpponentAnalysisService::new(
                                        client,
                                    );
                                let ck = crate::config::consumer_key();
                                let cs = crate::config::consumer_secret();

                                let get_auth = || {
                                    crate::chpp::oauth::create_oauth_context(
                                        &ck, &cs, &token, &secret,
                                    )
                                };

                                if let Ok(players) = service
                                    .get_opponent_match_lineup(&get_auth, match_id, team_id)
                                    .await
                                {
                                    // Clear existing pitch
                                    while let Some(child) = pitch_container_clone.first_child() {
                                        pitch_container_clone.remove(&child);
                                    }

                                    // Render new pitch
                                    let pitch =
                                        crate::ui::components::pitch_view::PitchView::create(
                                            &players,
                                        );
                                    pitch_container_clone.append(&pitch);
                                }
                            }
                        });
                    }
                }
            });

            // Bind selected opponent item to property
            self.dropdown_team
                .bind_property("selected-item", &*self.obj(), "selected-opponent")
                .sync_create()
                .build();

            // Preload matches from DB when opponent is selected (reactive property notification)
            let obj_for_notify = self.obj().clone();
            self.obj()
                .connect_notify_local(Some("selected-opponent"), move |obj, _| {
                    obj.handle_opponent_selected();
                });

            // Handle match selection

            self.obj().setup_handlers();
        }
    }

    impl WidgetImpl for OpponentAnalysis {}
    impl BoxImpl for OpponentAnalysis {}
}

glib::wrapper! {
    pub struct OpponentAnalysis(ObjectSubclass<imp::OpponentAnalysis>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl OpponentAnalysis {
    pub fn new() -> Self {
        glib::Object::builder()
            .property("orientation", gtk::Orientation::Vertical)
            .build()
    }

    fn handle_opponent_selected(&self) {
        let item = self.property::<Option<model::OpponentItem>>("selected-opponent");
        let team_id = if let Some(i) = item {
            i.team_id()
        } else {
            return;
        };

        let self_clone = self.clone();
        glib::MainContext::default().spawn_local(async move {
            let client = std::sync::Arc::new(crate::chpp::client::HattrickClient::new());
            let service = OpponentAnalysisService::new(client);
            let local_imp = self_clone.imp();

            // 1. Load stored ratings (the tactical analysis summary)
            let stored = service
                .get_stored_match_ratings(team_id)
                .unwrap_or_default();

            if !stored.is_empty() {
                self_clone.update_tactical_analysis(&stored);
            } else {
                while let Some(child) = local_imp.average_ratings_pitch.first_child() {
                    local_imp.average_ratings_pitch.remove(&child);
                }
                local_imp.lbl_formations.set_text("");
                local_imp.lbl_tactical_summary.set_text("");
            }

            // 2. Load matches from DB for the list view
            match service.get_latest_matches_from_db(team_id) {
                Ok(Some(matches_data)) => {
                    let finished: Vec<_> = matches_data
                        .Team
                        .MatchList
                        .Matches
                        .into_iter()
                        .filter(|m| m.Status == "FINISHED")
                        .map(|m| {
                            let is_home =
                                m.HomeTeam.HomeTeamID.parse::<u32>().unwrap_or(0) == team_id;
                            let rating = stored.iter().find(|r| r.match_id as u32 == m.MatchID);
                            crate::service::opponent_analysis::OpponentMatchData {
                                match_id: m.MatchID,
                                match_date: m.MatchDate,
                                is_home,
                                home_team_name: m.HomeTeam.HomeTeamName.clone(),
                                away_team_name: m.AwayTeam.AwayTeamName.clone(),
                                opponent_team_name: if is_home {
                                    m.AwayTeam.AwayTeamName
                                } else {
                                    m.HomeTeam.HomeTeamName
                                },
                                match_type: m.MatchType,
                                home_goals: m.HomeGoals,
                                away_goals: m.AwayGoals,
                                formation: rating.and_then(|r| r.formation.clone()),
                                tactic_type: rating.and_then(|r| r.tactic_type.map(|t| t as u32)),
                                rating_midfield: rating
                                    .and_then(|r| r.rating_midfield.map(|v| v as u32)),
                                rating_right_def: rating
                                    .and_then(|r| r.rating_right_def.map(|v| v as u32)),
                                rating_mid_def: rating
                                    .and_then(|r| r.rating_mid_def.map(|v| v as u32)),
                                rating_left_def: rating
                                    .and_then(|r| r.rating_left_def.map(|v| v as u32)),
                                rating_right_att: rating
                                    .and_then(|r| r.rating_right_att.map(|v| v as u32)),
                                rating_mid_att: rating
                                    .and_then(|r| r.rating_mid_att.map(|v| v as u32)),
                                rating_left_att: rating
                                    .and_then(|r| r.rating_left_att.map(|v| v as u32)),
                            }
                        })
                        .collect();

                    local_imp.latest_matches.replace(finished);
                }
                _ => {
                    local_imp.latest_matches.replace(Vec::new());
                }
            }
            self_clone.populate_match_list();
        });
    }

    fn update_tactical_analysis(&self, stored: &[crate::db::match_ratings::MatchRating]) {
        let imp = self.imp();
        let avg_pitch = imp.average_ratings_pitch.clone();
        let lbl_formations = imp.lbl_formations.clone();
        let lbl_tactical_summary = imp.lbl_tactical_summary.clone();

        if !stored.is_empty() {
            let n = stored.len() as f64;

            // Calculate averages
            let mid_avg = stored.iter().filter_map(|r| r.rating_midfield).sum::<f64>() / n;
            let rd_avg = stored
                .iter()
                .filter_map(|r| r.rating_right_def)
                .sum::<f64>()
                / n;
            let cd_avg = stored.iter().filter_map(|r| r.rating_mid_def).sum::<f64>() / n;
            let ld_avg = stored.iter().filter_map(|r| r.rating_left_def).sum::<f64>() / n;
            let ra_avg = stored
                .iter()
                .filter_map(|r| r.rating_right_att)
                .sum::<f64>()
                / n;
            let ca_avg = stored.iter().filter_map(|r| r.rating_mid_att).sum::<f64>() / n;
            let la_avg = stored.iter().filter_map(|r| r.rating_left_att).sum::<f64>() / n;

            // Calculate Formation Frequencies
            let mut formation_counts = std::collections::HashMap::new();
            for r in stored {
                if let Some(f) = &r.formation {
                    *formation_counts.entry(f.clone()).or_insert(0) += 1;
                }
            }
            let mut frequencies: Vec<_> = formation_counts.into_iter().collect();
            frequencies.sort_by(|a, b| b.1.cmp(&a.1));

            let mut formations_str = String::new();
            for (f, count) in frequencies {
                let percent = (count as f64 / n * 100.0).round();
                formations_str.push_str(&format!("{}: {} ({}%)\n", f, count, percent));
            }
            lbl_formations.set_text(&formations_str);

            // Identify strongest/weakest sectors
            // Sectors: 0:LD, 1:CD, 2:RD, 3:LA, 4:CA, 5:RA
            let def_sectors = [
                ("Left Defense", ld_avg),
                ("Central Defense", cd_avg),
                ("Right Defense", rd_avg),
            ];
            let att_sectors = [
                ("Left Attack", la_avg),
                ("Central Attack", ca_avg),
                ("Right Attack", ra_avg),
            ];

            let weakest_def = def_sectors
                .iter()
                .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .unwrap();
            let strongest_att = att_sectors
                .iter()
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
                .unwrap();

            let summary = format!(
                "<b>{}</b>: {} ({:.1})\n<b>{}</b>: {} ({:.1})",
                gettext("Weakest Defense"),
                gettext(weakest_def.0),
                weakest_def.1,
                gettext("Strongest Attack"),
                gettext(strongest_att.0),
                strongest_att.1
            );
            lbl_tactical_summary.set_markup(&summary);

            // Update Pitch Visualization
            use crate::ui::components::sector_ratings_view::SectorRatingsView;
            let ratings = [la_avg, ca_avg, ra_avg, mid_avg, ld_avg, cd_avg, rd_avg];
            let sector_view = SectorRatingsView::create(&ratings);

            if let Some(ctx) = imp.context.borrow().as_ref() {
                let ratings_f32 = [
                    la_avg as f32,
                    ca_avg as f32,
                    ra_avg as f32,
                    mid_avg as f32,
                    ld_avg as f32,
                    cd_avg as f32,
                    rd_avg as f32,
                ];
                ctx.set_opponent_avg_ratings(Some(ratings_f32));
            }

            while let Some(child) = avg_pitch.first_child() {
                avg_pitch.remove(&child);
            }
            avg_pitch.append(&sector_view);
        } else {
            lbl_formations.set_text(&gettext("No data yet."));
            lbl_tactical_summary.set_text(&gettext("No data yet."));
            while let Some(child) = avg_pitch.first_child() {
                avg_pitch.remove(&child);
            }
        }
    }

    fn on_filters_changed(&self) {
        let item = self.property::<Option<model::OpponentItem>>("selected-opponent");
        let team_id = if let Some(i) = item {
            i.team_id()
        } else {
            return;
        };

        let imp = self.imp();
        let limit = imp.spin_limit.value_as_int() as usize;

        // Build match type filter from checkboxes
        let mut match_type_filter: Vec<u32> = Vec::new();
        if imp.check_league.is_active() {
            match_type_filter.push(1); // League
            match_type_filter.push(2); // Qualifier
        }
        if imp.check_cup.is_active() {
            match_type_filter.push(3); // Cup
            match_type_filter.push(7); // Masters Cup
            match_type_filter.push(8); // International Cup
        }
        if imp.check_friendly.is_active() {
            match_type_filter.push(4); // Friendly by challenge
            match_type_filter.push(5); // Friendly international
        }
        if imp.check_international.is_active() {
            match_type_filter.push(7); // Masters Cup
            match_type_filter.push(8); // International Cup
        }

        let self_clone = self.clone();
        glib::MainContext::default().spawn_local(async move {
            let client = std::sync::Arc::new(crate::chpp::client::HattrickClient::new());
            let service = OpponentAnalysisService::new(client);
            let local_imp = self_clone.imp();

            // Load matches from DB
            if let Ok(Some(matches_data)) = service.get_latest_matches_from_db(team_id) {
                let finished: Vec<_> = matches_data
                    .Team
                    .MatchList
                    .Matches
                    .into_iter()
                    .filter(|m| m.Status == "FINISHED")
                    .filter(|m| {
                        if match_type_filter.is_empty() {
                            true
                        } else {
                            match_type_filter.contains(&m.MatchType)
                        }
                    })
                    .collect();

                let stored = service
                    .get_stored_match_ratings(team_id)
                    .unwrap_or_default();

                let mut mapped_matches: Vec<_> = finished
                    .into_iter()
                    .map(|m| {
                        let is_home = m.HomeTeam.HomeTeamID.parse::<u32>().unwrap_or(0) == team_id;
                        let rating = stored.iter().find(|r| r.match_id as u32 == m.MatchID);
                        crate::service::opponent_analysis::OpponentMatchData {
                            match_id: m.MatchID,
                            match_date: m.MatchDate,
                            is_home,
                            home_team_name: m.HomeTeam.HomeTeamName.clone(),
                            away_team_name: m.AwayTeam.AwayTeamName.clone(),
                            opponent_team_name: if is_home {
                                m.AwayTeam.AwayTeamName
                            } else {
                                m.HomeTeam.HomeTeamName
                            },
                            match_type: m.MatchType,
                            home_goals: m.HomeGoals,
                            away_goals: m.AwayGoals,
                            formation: rating.and_then(|r| r.formation.clone()),
                            tactic_type: rating.and_then(|r| r.tactic_type.map(|t| t as u32)),
                            rating_midfield: rating
                                .and_then(|r| r.rating_midfield.map(|v| v as u32)),
                            rating_right_def: rating
                                .and_then(|r| r.rating_right_def.map(|v| v as u32)),
                            rating_mid_def: rating.and_then(|r| r.rating_mid_def.map(|v| v as u32)),
                            rating_left_def: rating
                                .and_then(|r| r.rating_left_def.map(|v| v as u32)),
                            rating_right_att: rating
                                .and_then(|r| r.rating_right_att.map(|v| v as u32)),
                            rating_mid_att: rating.and_then(|r| r.rating_mid_att.map(|v| v as u32)),
                            rating_left_att: rating
                                .and_then(|r| r.rating_left_att.map(|v| v as u32)),
                        }
                    })
                    .take(limit)
                    .collect();

                local_imp.latest_matches.replace(mapped_matches);

                // If we don't have enough matches AND it's not because they haven't played enough yet..
                // For safety, let the Analysis button fetch from the API. The user requested automatic fetching on DB limit miss.
                let db_count = local_imp.latest_matches.borrow().len();
                if db_count < limit {
                    // Try fetch from API in background quietly
                    let secret_service = crate::service::secret::SystemSecretService::new();
                    use crate::service::secret::SecretStorageService;
                    if let (Ok(Some(token)), Ok(Some(secret))) = (
                        secret_service.get_secret("access_token").await,
                        secret_service.get_secret("access_secret").await,
                    ) {
                        let filter_clone = if match_type_filter.is_empty() {
                            None
                        } else {
                            Some(match_type_filter.clone())
                        };
                        let ck = crate::config::consumer_key();
                        let cs = crate::config::consumer_secret();
                        let get_auth =
                            || crate::chpp::oauth::create_oauth_context(&ck, &cs, &token, &secret);
                        if let Ok(analysis) = service
                            .analyze_opponent(&get_auth, team_id, limit, filter_clone)
                            .await
                        {
                            local_imp.latest_matches.replace(analysis.matches);
                        }
                    }
                }
            } else {
                local_imp.latest_matches.replace(Vec::new());
            }
            self_clone.populate_match_list();
        });
    }

    fn populate_match_list(&self) {
        let imp = self.imp();
        let matches = imp.latest_matches.borrow();

        // Already filtered in on_filters_changed
        let filtered: Vec<_> = matches.clone();

        if let Some(selection_model) = imp.match_list_view.model() {
            if let Some(list_model) = selection_model.downcast_ref::<gtk::SingleSelection>() {
                if let Some(store) = list_model.model() {
                    if let Some(list_store) = store.downcast_ref::<gio::ListStore>() {
                        list_store.remove_all();

                        for m in filtered {
                            let item = model::MatchItem::new(
                                m.match_id,
                                &m.match_date,
                                m.is_home,
                                &m.home_team_name,
                                &m.away_team_name,
                                &m.opponent_team_name,
                                m.match_type,
                                m.home_goals,
                                m.away_goals,
                                m.formation.clone(),
                                m.tactic_type,
                                m.rating_midfield,
                                m.rating_right_def,
                                m.rating_mid_def,
                                m.rating_left_def,
                                m.rating_right_att,
                                m.rating_mid_att,
                                m.rating_left_att,
                            );
                            list_store.append(&item);
                        }
                    }
                }
            }
        }
    }

    fn setup_handlers(&self) {
        let imp = self.imp();
        let btn = imp.btn_analyze.clone();
        let dropdown = imp.dropdown_team.clone();
        let spin = imp.spin_limit.clone();
        let spinner = imp.spinner.clone();
        let lbl_formations = imp.lbl_formations.clone();
        let lbl_unavailable = imp.lbl_unavailable.clone();
        let match_list_view = imp.match_list_view.clone();
        let pitch_container = imp.pitch_container.clone();
        let check_league = imp.check_league.clone();
        let check_cup = imp.check_cup.clone();
        let check_friendly = imp.check_friendly.clone();
        let check_international = imp.check_international.clone();
        let obj_clone = self.clone();

        // Connect filter toggles to refresh list
        let obj_filter = self.clone();
        check_league.connect_toggled(move |_| obj_filter.on_filters_changed());
        let obj_filter = self.clone();
        check_cup.connect_toggled(move |_| obj_filter.on_filters_changed());
        let obj_filter = self.clone();
        check_friendly.connect_toggled(move |_| obj_filter.on_filters_changed());
        let obj_filter = self.clone();
        check_international.connect_toggled(move |_| obj_filter.on_filters_changed());
        let obj_filter = self.clone();
        spin.connect_value_changed(move |_| obj_filter.on_filters_changed());

        btn.connect_clicked(move |btn_ref| {
            let selected_item = dropdown.selected_item();
            let team_id = if let Some(item) = selected_item.and_downcast::<model::OpponentItem>() {
                item.team_id()
            } else {
                lbl_formations.set_text(&gettext("No opponent selected"));
                return;
            };

            // Build match type filter from checkboxes
            let mut match_type_filter: Vec<u32> = Vec::new();
            if check_league.is_active() {
                match_type_filter.push(1); // League
                match_type_filter.push(2); // Qualifier
            }
            if check_cup.is_active() {
                match_type_filter.push(3); // Cup
                match_type_filter.push(7); // Masters Cup
                match_type_filter.push(8); // International Cup
            }
            if check_friendly.is_active() {
                match_type_filter.push(4); // Friendly by challenge
                match_type_filter.push(5); // Friendly international
            }
            if check_international.is_active() {
                match_type_filter.push(7); // Masters Cup
                match_type_filter.push(8); // International Cup
            }
            let match_type_filter = if match_type_filter.is_empty() {
                None
            } else {
                match_type_filter.sort_unstable();
                match_type_filter.dedup();
                Some(match_type_filter)
            };

            let limit = spin.value_as_int() as usize;

            spinner.set_spinning(true);
            btn_ref.set_sensitive(false);

            let lbl_formations_clone = lbl_formations.clone();
            let lbl_unavailable_clone = lbl_unavailable.clone();
            let spinner_clone = spinner.clone();
            let btn_clone = btn_ref.clone();
            let pitch_container_clone = pitch_container.clone();
            let local_obj = obj_clone.clone();

            glib::MainContext::default().spawn_local(async move {
                let local_imp = local_obj.imp();
                let secret_service = SystemSecretService::new();
                use crate::service::secret::SecretStorageService;

                let token_res = secret_service.get_secret("access_token").await;
                let secret_res = secret_service.get_secret("access_secret").await;

                if let (Ok(Some(token)), Ok(Some(secret))) = (token_res, secret_res) {
                    let client = Arc::new(HattrickClient::new());
                    let service = OpponentAnalysisService::new(client);
                    let ck = consumer_key();
                    let cs = consumer_secret();

                    let get_auth =
                        || crate::chpp::oauth::create_oauth_context(&ck, &cs, &token, &secret);

                    match service
                        .analyze_opponent(&get_auth, team_id, limit, match_type_filter)
                        .await
                    {
                        Ok(analysis) => {
                            // Clear existing pitch when loading a new team
                            while let Some(child) = pitch_container_clone.first_child() {
                                pitch_container_clone.remove(&child);
                            }

                            // Format Formations
                            let mut formations_str = String::new();
                            for (f, count) in &analysis.formation_frequencies {
                                let f_text = format!("{}: {} times\n", f, count);
                                formations_str.push_str(&f_text);
                            }
                            if formations_str.is_empty() {
                                formations_str = "None found.".to_string();
                            }
                            lbl_formations_clone.set_text(&formations_str);

                            // Format Unavailable Players
                            let mut unav_str = String::new();
                            for p in &analysis.injured_or_suspended_players {
                                let reason = if p.InjuryLevel.unwrap_or(-1) > 0 {
                                    "Injured"
                                } else {
                                    "Suspended"
                                };
                                let line = format!("{} {} ({})\n", p.FirstName, p.LastName, reason);
                                unav_str.push_str(&line);
                            }
                            if unav_str.is_empty() {
                                unav_str = gettext("None.");
                            }
                            lbl_unavailable_clone.set_text(&unav_str);

                            // Format Averages using PitchView
                            let matches_count = analysis.matches.len() as f64;
                            if matches_count > 0.0 {
                                let mut mid = 0.0;
                                let mut rd = 0.0;
                                let mut cd = 0.0;
                                let mut ld = 0.0;
                                let mut ra = 0.0;
                                let mut ca = 0.0;
                                let mut la = 0.0;
                                for m in &analysis.matches {
                                    mid += m.rating_midfield.unwrap_or(0) as f64;
                                    rd += m.rating_right_def.unwrap_or(0) as f64;
                                    cd += m.rating_mid_def.unwrap_or(0) as f64;
                                    ld += m.rating_left_def.unwrap_or(0) as f64;
                                    ra += m.rating_right_att.unwrap_or(0) as f64;
                                    ca += m.rating_mid_att.unwrap_or(0) as f64;
                                    la += m.rating_left_att.unwrap_or(0) as f64;
                                }

                                let mid_avg = mid / matches_count;
                                let rd_avg = rd / matches_count;
                                let cd_avg = cd / matches_count;
                                let ld_avg = ld / matches_count;
                                let ra_avg = ra / matches_count;
                                let ca_avg = ca / matches_count;
                                let la_avg = la / matches_count;

                                // Build sector ratings visualization
                                use crate::ui::components::sector_ratings_view::SectorRatingsView;
                                let ratings =
                                    [la_avg, ca_avg, ra_avg, mid_avg, ld_avg, cd_avg, rd_avg];
                                let sector_view = SectorRatingsView::create(&ratings);

                                if let Some(ctx) = local_imp.context.borrow().as_ref() {
                                    let ratings_f32 = [
                                        la_avg as f32,
                                        ca_avg as f32,
                                        ra_avg as f32,
                                        mid_avg as f32,
                                        ld_avg as f32,
                                        cd_avg as f32,
                                        rd_avg as f32,
                                    ];
                                    ctx.set_opponent_avg_ratings(Some(ratings_f32));
                                }

                                while let Some(child) =
                                    local_imp.average_ratings_pitch.first_child()
                                {
                                    local_imp.average_ratings_pitch.remove(&child);
                                }
                                local_imp.average_ratings_pitch.append(&sector_view);

                                // Persist ratings to DB
                                use crate::db::match_ratings::NewMatchRating;
                                let download_id = crate::db::manager::DbManager::new()
                                    .get_connection()
                                    .ok()
                                    .and_then(|mut conn| {
                                        crate::db::download_entries::create_download(
                                            &mut conn,
                                            &chrono::Utc::now().to_rfc3339(),
                                            "completed",
                                        )
                                        .ok()
                                    })
                                    .unwrap_or(0);
                                let new_ratings: Vec<NewMatchRating> = analysis
                                    .matches
                                    .iter()
                                    .filter(|m| m.rating_midfield.is_some())
                                    .map(|m| NewMatchRating {
                                        match_id: m.match_id as i32,
                                        team_id: team_id as i32,
                                        download_id,
                                        formation: m.formation.clone(),
                                        tactic_type: m.tactic_type.map(|t| t as i32),
                                        rating_midfield: m.rating_midfield.map(|v| v as f64),
                                        rating_right_def: m.rating_right_def.map(|v| v as f64),
                                        rating_mid_def: m.rating_mid_def.map(|v| v as f64),
                                        rating_left_def: m.rating_left_def.map(|v| v as f64),
                                        rating_right_att: m.rating_right_att.map(|v| v as f64),
                                        rating_mid_att: m.rating_mid_att.map(|v| v as f64),
                                        rating_left_att: m.rating_left_att.map(|v| v as f64),
                                    })
                                    .collect();

                                if !new_ratings.is_empty() {
                                    let _ = service.save_match_ratings(&new_ratings);
                                }

                                if let Ok(stored_ratings) =
                                    service.get_stored_match_ratings(team_id)
                                {
                                    local_obj.update_tactical_analysis(&stored_ratings);
                                }

                                local_imp.latest_matches.replace(analysis.matches.clone());
                                local_obj.populate_match_list();

                                // Calculate and display the best counter formation
                                if let Some(ctx) = local_imp.context.borrow().as_ref() {
                                    if let Some(lineups) = ctx.best_lineups() {
                                        use crate::rating::types::RatingSector;

                                        let mut best_lineup_name = String::new();
                                        let mut best_score = -9999.0;

                                        for lineup in lineups {
                                            let m = lineup
                                                .sector_ratings
                                                .get(&RatingSector::Midfield)
                                                .unwrap_or(&0.0)
                                                - mid_avg;

                                            let al = lineup
                                                .sector_ratings
                                                .get(&RatingSector::AttackLeft)
                                                .unwrap_or(&0.0)
                                                - rd_avg;
                                            let ac = lineup
                                                .sector_ratings
                                                .get(&RatingSector::AttackCentral)
                                                .unwrap_or(&0.0)
                                                - cd_avg;
                                            let ar = lineup
                                                .sector_ratings
                                                .get(&RatingSector::AttackRight)
                                                .unwrap_or(&0.0)
                                                - ld_avg;

                                            let dl = lineup
                                                .sector_ratings
                                                .get(&RatingSector::DefenceLeft)
                                                .unwrap_or(&0.0)
                                                - ra_avg;
                                            let dc = lineup
                                                .sector_ratings
                                                .get(&RatingSector::DefenceCentral)
                                                .unwrap_or(&0.0)
                                                - ca_avg;
                                            let dr = lineup
                                                .sector_ratings
                                                .get(&RatingSector::DefenceRight)
                                                .unwrap_or(&0.0)
                                                - la_avg;

                                            let score = (m * 3.0) + al + ac + ar + dl + dc + dr;

                                            if score > best_score {
                                                best_score = score;
                                                best_lineup_name =
                                                    lineup.formation.name().to_string();
                                            }
                                        }

                                        if !best_lineup_name.is_empty() {
                                            let current =
                                                local_imp.lbl_tactical_summary.label().to_string();
                                            let summary = format!(
                                                "{}\n\n<b>{}</b>: {}",
                                                current,
                                                gettext("Recommended Counter"),
                                                best_lineup_name
                                            );
                                            local_imp.lbl_tactical_summary.set_markup(&summary);
                                        }
                                    }
                                }
                            } else {
                                local_obj.update_tactical_analysis(&[]);
                                let msg = gtk::Label::new(Some(&gettext("No matches found.")));
                                local_imp.average_ratings_pitch.append(&msg);
                            }

                            local_imp.latest_matches.replace(analysis.matches);
                            local_obj.populate_match_list();
                        }
                        Err(_e) => {
                            let msg = "Error communicating with CHPP API".to_string();
                            lbl_formations_clone.set_text(&msg);
                        }
                    }
                } else {
                    lbl_formations_clone.set_text(&gettext("OAuth secrets not found!"));
                }
                spinner_clone.set_spinning(false);
                btn_clone.set_sensitive(true);
            });
        });
    }
}

impl Default for OpponentAnalysis {
    fn default() -> Self {
        Self::new()
    }
}
