/* mod.rs
 *
 * Copyright 2026 Sébastien Le Callonnec
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, Button, Label, SpinButton, Spinner};
use log::error;
use std::sync::Arc;

use crate::chpp::client::HattrickClient;
use crate::config::{consumer_key, consumer_secret};
use crate::service::opponent_analysis::OpponentAnalysisService;
use crate::service::secret::SystemSecretService;

mod model;

mod imp {
    use super::*;
    use gtk::CompositeTemplate;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/Nutmeg/ui/opponent_analysis/opponent_analysis.ui")]
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
        pub lbl_ratings: TemplateChild<Label>,
        #[template_child]
        pub lbl_unavailable: TemplateChild<Label>,
        #[template_child]
        pub match_list_view: TemplateChild<gtk::ListView>,
        #[template_child]
        pub pitch_container: TemplateChild<gtk::Box>,

        pub selected_team: std::cell::RefCell<Option<crate::ui::team_object::TeamObject>>,
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
                    glib::ParamSpecObject::builder::<crate::ui::team_object::TeamObject>("selected-team")
                        .explicit_notify()
                        .build(),
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "selected-team" => self.selected_team.borrow().to_value(),
                _ => unimplemented!(),
            }
        }

        fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
            match pspec.name() {
                "selected-team" => {
                    let team = value.get::<Option<crate::ui::team_object::TeamObject>>().expect("Value must be Option<TeamObject>");
                    self.selected_team.replace(team);
                    self.obj().notify("selected-team");
                    self.obj().fetch_upcoming_opponents();
                }
                _ => unimplemented!(),
            }
        }

        fn constructed(&self) {
            self.parent_constructed();

            // Set an expression to extract display-text for the DropDown
            let expression = gtk::PropertyExpression::new(
                super::model::OpponentItem::static_type(),
                None::<&gtk::Expression>,
                "display-text",
            );
            self.dropdown_team.set_expression(Some(expression));

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
                let lbl_desc = gtk::Label::builder()
                    .halign(gtk::Align::Start)
                    .css_classes(["dim-label"])
                    .build();

                vbox.append(&lbl_date);
                vbox.append(&lbl_desc);

                list_item.set_child(Some(&vbox));
            });
            factory.connect_bind(|_, list_item| {
                let list_item = list_item
                    .downcast_ref::<gtk::ListItem>()
                    .expect("Needs to be ListItem");
                let match_item = list_item
                    .item()
                    .and_downcast::<super::model::MatchItem>()
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
                let lbl_desc = vbox.last_child().unwrap().downcast::<gtk::Label>().unwrap();

                let is_home_str = if match_item.is_home() { "Home" } else { "Away" };
                let hg = match_item.home_goals();
                let ag = match_item.away_goals();
                let result_str = if let (Some(h), Some(a)) = (hg, ag) {
                    format!("{} - {}", h, a)
                } else {
                    "?-?".to_string()
                };

                let form_str = match_item
                    .formation()
                    .unwrap_or_else(|| "Unknown".to_string());

                lbl_date.set_text(&match_item.match_date());
                lbl_desc.set_text(&format!("{} | {} | {}", is_home_str, result_str, form_str));
            });

            self.match_list_view.set_factory(Some(&factory));

            let store = gio::ListStore::new::<super::model::MatchItem>();
            let selection_model = gtk::SingleSelection::new(Some(store));
            self.match_list_view.set_model(Some(&selection_model));

            // Handle match selection
            let pitch_container = self.pitch_container.clone();
            let team_dropdown = self.dropdown_team.clone();
            selection_model.connect_selected_item_notify(move |sel| {
                if let Some(item) = sel.selected_item() {
                    if let Ok(match_item) = item.downcast::<super::model::MatchItem>() {
                        let match_id = match_item.match_id();

                        let selected_team = team_dropdown.selected_item();
                        let team_id = if let Some(t) =
                            selected_team.and_downcast::<super::model::OpponentItem>()
                        {
                            t.team_id()
                        } else {
                            return;
                        };

                        let pitch_container_clone = pitch_container.clone();

                        glib::MainContext::default().spawn_local(async move {
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

            self.obj().setup_handlers();
            // Initial fetch will be triggered by the selected-team property binding
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

    fn setup_handlers(&self) {
        let imp = self.imp();
        let btn = imp.btn_analyze.clone();
        let dropdown = imp.dropdown_team.clone();
        let spin = imp.spin_limit.clone();
        let spinner = imp.spinner.clone();
        let lbl_formations = imp.lbl_formations.clone();
        let lbl_ratings = imp.lbl_ratings.clone();
        let lbl_unavailable = imp.lbl_unavailable.clone();
        let match_list_view = imp.match_list_view.clone();
        let pitch_container = imp.pitch_container.clone();

        btn.connect_clicked(move |btn_ref| {
            let selected_item = dropdown.selected_item();
            let team_id = if let Some(item) = selected_item.and_downcast::<model::OpponentItem>() {
                item.team_id()
            } else {
                lbl_formations.set_text("No opponent selected");
                return;
            };

            let limit = spin.value_as_int() as usize;

            spinner.set_spinning(true);
            btn_ref.set_sensitive(false);

            let lbl_formations_clone = lbl_formations.clone();
            let lbl_ratings_clone = lbl_ratings.clone();
            let lbl_unavailable_clone = lbl_unavailable.clone();
            let spinner_clone = spinner.clone();
            let btn_clone = btn_ref.clone();
            let match_list_view_clone = match_list_view.clone();
            let pitch_container_clone = pitch_container.clone();

            glib::MainContext::default().spawn_local(async move {
                let secret_service = SystemSecretService::new();
                use crate::service::secret::SecretStorageService;

                let token_res = secret_service.get_secret("access_token").await;
                let secret_res = secret_service.get_secret("access_secret").await;

                if let (Ok(Some(token)), Ok(Some(secret))) = (token_res, secret_res) {
                    let client = Arc::new(HattrickClient::new());
                    let service = OpponentAnalysisService::new(client);
                    let ck = consumer_key();
                    let cs = consumer_secret();

                    let get_auth = || {
                        crate::chpp::oauth::create_oauth_context(
                            &ck,
                            &cs,
                            &token,
                            &secret,
                        )
                    };

                    match service.analyze_opponent(&get_auth, team_id, limit).await {
                        Ok(analysis) => {
                            // Populate list model
                            if let Some(selection_model) = match_list_view_clone.model() {
                                if let Some(list_model) = selection_model.downcast_ref::<gtk::SingleSelection>() {
                                    if let Some(store) = list_model.model() {
                                        if let Some(list_store) = store.downcast_ref::<gio::ListStore>() {
                                            list_store.remove_all();

                                            // Clear existing pitch when loading a new team
                                            while let Some(child) = pitch_container_clone.first_child() {
                                                pitch_container_clone.remove(&child);
                                            }

                                            for m in &analysis.matches {
                                                let item = model::MatchItem::new(
                                                    m.match_id,
                                                    &m.match_date,
                                                    m.is_home,
                                                    m.match_type,
                                                    m.home_goals,
                                                    m.away_goals,
                                                    m.formation.clone(),
                                                );
                                                list_store.append(&item);
                                            }
                                        }
                                    }
                                }
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
                                let reason = if p.InjuryLevel.unwrap_or(-1) > 0 { "Injured" } else { "Suspended" };
                                let line = format!("{} {} ({})\n", p.FirstName, p.LastName, reason);
                                unav_str.push_str(&line);
                            }
                            if unav_str.is_empty() {
                                unav_str = "None.".to_string();
                            }
                            lbl_unavailable_clone.set_text(&unav_str);

                            // Format Averages
                            let matches_count = analysis.matches.len() as u32;
                            if matches_count > 0 {
                                let mut mid = 0; let mut rd = 0; let mut cd = 0; let mut ld = 0;
                                let mut ra = 0; let mut ca = 0; let mut la = 0;
                                for m in &analysis.matches {
                                    mid += m.rating_midfield.unwrap_or(0);
                                    rd += m.rating_right_def.unwrap_or(0);
                                    cd += m.rating_mid_def.unwrap_or(0);
                                    ld += m.rating_left_def.unwrap_or(0);
                                    ra += m.rating_right_att.unwrap_or(0);
                                    ca += m.rating_mid_att.unwrap_or(0);
                                    la += m.rating_left_att.unwrap_or(0);
                                }
                                let avg_str = format!(
                                    "Midfield: {:.1}\nDefence (L/C/R): {:.1} / {:.1} / {:.1}\nAttack (L/C/R): {:.1} / {:.1} / {:.1}",
                                    mid as f32 / matches_count as f32,
                                    ld as f32 / matches_count as f32,
                                    cd as f32 / matches_count as f32,
                                    rd as f32 / matches_count as f32,
                                    la as f32 / matches_count as f32,
                                    ca as f32 / matches_count as f32,
                                    ra as f32 / matches_count as f32,
                                );
                                lbl_ratings_clone.set_text(&avg_str);
                            } else {
                                lbl_ratings_clone.set_text("No matches found.");
                            }
                        },
                        Err(_e) => {
                            let msg = "Error communicating with CHPP API".to_string();
                            lbl_formations_clone.set_text(&msg);
                        }
                    }
                } else {
                    lbl_formations_clone.set_text("OAuth secrets not found!");
                }

                spinner_clone.set_spinning(false);
                btn_clone.set_sensitive(true);
            });
        });
    }

    fn fetch_upcoming_opponents(&self) {
        let team_obj = self.property::<Option<crate::ui::team_object::TeamObject>>("selected-team");
        let our_team_id = if let Some(t) = team_obj {
            t.team_data().id
        } else {
            return;
        };

        let dropdown = self.imp().dropdown_team.clone();
        let list_store = gio::ListStore::new::<model::OpponentItem>();
        dropdown.set_model(Some(&list_store));

        glib::MainContext::default().spawn_local(async move {
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

                match service.get_upcoming_opponents(&get_auth, our_team_id).await {
                    Ok(opponents) => {
                        for opp in opponents {
                            let item = model::OpponentItem::new(
                                opp.team_id,
                                &opp.team_name,
                                &opp.match_date,
                            );
                            list_store.append(&item);
                        }
                    }
                    Err(_e) => {
                        error!("Failed to fetch upcoming opponents: {}", _e);
                    }
                }
            }
        });
    }
}

impl Default for OpponentAnalysis {
    fn default() -> Self {
        Self::new()
    }
}
