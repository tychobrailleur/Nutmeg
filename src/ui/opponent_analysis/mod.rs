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
use gettextrs::gettext;
use gio::prelude::*;
use glib::Object;

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

        pub selected_team: std::cell::RefCell<Option<crate::ui::team_object::TeamObject>>,
        pub context: std::cell::RefCell<Option<crate::ui::context_object::ContextObject>>,
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
                ]
            })
        }

        fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
            match pspec.name() {
                "selected-team" => self.selected_team.borrow().to_value(),
                "context" => self.context.borrow().to_value(),
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
                    self.obj().fetch_upcoming_opponents();
                }
                "context" => {
                    let ctx = value
                        .get::<Option<crate::ui::context_object::ContextObject>>()
                        .expect("Value must be Option<ContextObject>");
                    self.context.replace(ctx);
                    self.obj().notify("context");
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
                    "{} {} – {} {}",
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

                lbl_date.set_text(&match_item.match_date());
                lbl_score.set_text(&score_str);
                lbl_desc.set_text(&format!("{} | {}", tactic_name, form_str));

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

            // Preload matches from DB when opponent is selected in the dropdown
            let match_list_view_preload = self.match_list_view.clone();
            let avg_pitch_preload = self.average_ratings_pitch.clone();
            let obj_for_preload = self.obj().clone();
            self.dropdown_team
                .connect_selected_item_notify(move |dropdown| {
                    let selected = dropdown.selected_item();
                    let team_id =
                        if let Some(item) = selected.and_downcast::<super::model::OpponentItem>() {
                            item.team_id()
                        } else {
                            return;
                        };

                    let match_list_view_clone = match_list_view_preload.clone();
                    let avg_pitch_clone = avg_pitch_preload.clone();
                    let _obj_clone = obj_for_preload.clone();

                    glib::MainContext::default().spawn_local(async move {
                        let db_manager = crate::db::manager::DbManager::new();
                        if let Ok(mut conn) = db_manager.get_connection() {
                            if let Ok(Some(matches_data)) =
                                crate::db::series::get_latest_matches(&mut conn, team_id)
                            {
                                let finished: Vec<_> = matches_data
                                    .Team
                                    .MatchList
                                    .Matches
                                    .iter()
                                    .filter(|m| m.Status == "FINISHED")
                                    .cloned()
                                    .collect();

                                if !finished.is_empty() {
                                    if let Some(selection_model) = match_list_view_clone.model() {
                                        if let Some(single_sel) =
                                            selection_model.downcast_ref::<gtk::SingleSelection>()
                                        {
                                            if let Some(store) = single_sel.model() {
                                                if let Some(list_store) =
                                                    store.downcast_ref::<gio::ListStore>()
                                                {
                                                    list_store.remove_all();
                                                    for m in &finished {
                                                        let item = model::MatchItem::new(
                                                            m.MatchID,
                                                            &m.MatchDate,
                                                            m.HomeTeam
                                                                .HomeTeamID
                                                                .parse::<u32>()
                                                                .unwrap_or(0)
                                                                == team_id,
                                                            &m.HomeTeam.HomeTeamName,
                                                            &m.AwayTeam.AwayTeamName,
                                                            // opponent name: whoever is NOT team_id
                                                            if m.HomeTeam
                                                                .HomeTeamID
                                                                .parse::<u32>()
                                                                .unwrap_or(0)
                                                                == team_id
                                                            {
                                                                &m.AwayTeam.AwayTeamName
                                                            } else {
                                                                &m.HomeTeam.HomeTeamName
                                                            },
                                                            m.MatchType,
                                                            m.HomeGoals,
                                                            m.AwayGoals,
                                                            None, // formation
                                                            None, // tactic_type
                                                            None, // rating_midfield
                                                            None, // rating_right_def
                                                            None, // rating_mid_def
                                                            None, // rating_left_def
                                                            None, // rating_right_att
                                                            None, // rating_mid_att
                                                            None, // rating_left_att
                                                        );
                                                        list_store.append(&item);
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // Load stored ratings and compute sector viz from them
                                    if let Ok(stored_ratings) =
                                        crate::db::match_ratings::get_match_ratings(&mut conn, team_id)
                                    {
                                        use std::collections::HashMap;
                                        let ratings_map: HashMap<i32, &crate::db::match_ratings::MatchRating> =
                                            stored_ratings.iter().map(|r| (r.match_id, r)).collect();

                                        let matched: Vec<_> = finished
                                            .iter()
                                            .filter_map(|m| ratings_map.get(&(m.MatchID as i32)).copied())
                                            .collect();

                                        if !matched.is_empty() {
                                            let n = matched.len() as f64;
                                            let mid_avg = matched.iter().filter_map(|r| r.rating_midfield).sum::<f64>() / n;
                                            let rd_avg = matched.iter().filter_map(|r| r.rating_right_def).sum::<f64>() / n;
                                            let cd_avg = matched.iter().filter_map(|r| r.rating_mid_def).sum::<f64>() / n;
                                            let ld_avg = matched.iter().filter_map(|r| r.rating_left_def).sum::<f64>() / n;
                                            let ra_avg = matched.iter().filter_map(|r| r.rating_right_att).sum::<f64>() / n;
                                            let ca_avg = matched.iter().filter_map(|r| r.rating_mid_att).sum::<f64>() / n;
                                            let la_avg = matched.iter().filter_map(|r| r.rating_left_att).sum::<f64>() / n;

                                            use crate::ui::components::sector_ratings_view::SectorRatingsView;
                                            let ratings = [la_avg, ca_avg, ra_avg, mid_avg, ld_avg, cd_avg, rd_avg];
                                            let sector_view = SectorRatingsView::create(&ratings);

                                            if let Some(ctx) = _obj_clone.imp().context.borrow().as_ref() {
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

                                            while let Some(child) = avg_pitch_clone.first_child() {
                                                avg_pitch_clone.remove(&child);
                                            }
                                            avg_pitch_clone.append(&sector_view);
                                        } else {
                                            // No stored ratings yet — just clear the sector viz
                                            while let Some(child) = avg_pitch_clone.first_child() {
                                                avg_pitch_clone.remove(&child);
                                            }
                                        }
                                    } else {
                                        while let Some(child) = avg_pitch_clone.first_child() {
                                            avg_pitch_clone.remove(&child);
                                        }
                                    }
                                }
                            }
                        }
                    });
                });

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
            let match_list_view_clone = match_list_view.clone();
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
                            // Populate list model
                            if let Some(selection_model) = match_list_view_clone.model() {
                                if let Some(list_model) =
                                    selection_model.downcast_ref::<gtk::SingleSelection>()
                                {
                                    if let Some(store) = list_model.model() {
                                        if let Some(list_store) =
                                            store.downcast_ref::<gio::ListStore>()
                                        {
                                            list_store.remove_all();

                                            // Clear existing pitch when loading a new team
                                            while let Some(child) =
                                                pitch_container_clone.first_child()
                                            {
                                                pitch_container_clone.remove(&child);
                                            }

                                            for m in &analysis.matches {
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

                                // Clear existing average pitch
                                while let Some(child) =
                                    local_imp.average_ratings_pitch.first_child()
                                {
                                    local_imp.average_ratings_pitch.remove(&child);
                                }
                                local_imp.average_ratings_pitch.append(&sector_view);

                                // Persist ratings to DB for future preloads
                                let db_manager = crate::db::manager::DbManager::new();
                                if let Ok(mut conn) = db_manager.get_connection() {
                                    use crate::db::match_ratings::NewMatchRating;
                                    let new_ratings: Vec<NewMatchRating> = analysis
                                        .matches
                                        .iter()
                                        .filter(|m| m.rating_midfield.is_some())
                                        .map(|m| NewMatchRating {
                                            match_id: m.match_id as i32,
                                            team_id: team_id as i32,
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
                                        let _ = crate::db::match_ratings::save_match_ratings(
                                            &mut conn,
                                            &new_ratings,
                                        );
                                    }
                                }
                            } else {
                                while let Some(child) =
                                    local_imp.average_ratings_pitch.first_child()
                                {
                                    local_imp.average_ratings_pitch.remove(&child);
                                }
                                let msg = gtk::Label::new(Some(&gettext("No matches found.")));
                                local_imp.average_ratings_pitch.append(&msg);
                            }
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
