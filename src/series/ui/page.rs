/* series/ui/page.rs
 *
 * Copyright 2026 Sébastien Le Callonnec
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::chpp::model::{LeagueDetailsData, LeagueTeam, MatchDetails, MatchesData};
use gettextrs::gettext;
use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{
    glib, ColumnView, ColumnViewColumn, CompositeTemplate, DrawingArea, Label, ListItem,
    SignalListItemFactory,
};
use log::{debug, warn};

// ── Match outcome ──────────────────────────────────────────────────────────────

/// The result of a single league match from one team's perspective.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchOutcome {
    Win,
    Draw,
    Loss,
}

impl MatchOutcome {
    /// RGB colour used when drawing the result disc.
    fn colour(&self) -> (f64, f64, f64) {
        match self {
            MatchOutcome::Win => (0.22, 0.71, 0.29),  // green
            MatchOutcome::Draw => (0.85, 0.85, 0.85), // grey
            MatchOutcome::Loss => (0.87, 0.20, 0.20), // red
        }
    }
}

// ── Form entry ─────────────────────────────────────────────────────────────────

/// A single entry in a team's form bar: the match outcome plus the data needed
/// to build the hover tooltip ("Home Team 0 – 0 Away Team, date").
#[derive(Debug, Clone)]
pub struct FormEntry {
    pub outcome: MatchOutcome,
    pub match_date: String,
    pub home_team: String,
    pub away_team: String,
    pub home_goals: u32,
    pub away_goals: u32,
}

// ── Form helper ───────────────────────────────────────────────────────────────

/// Computes the form bar entries for `team_id`.
///
/// Only finished league matches are considered.  Results are scoped to the
/// **current season** by taking at most `season_match_count` of the most-recent
/// matches (relying on the invariant that the current season's games are always
/// the most recent ones in the dataset).  At most `max_results` entries are
/// returned so the bar never exceeds 5 discs.
///
/// The returned vec is **newest-first** so index 0 is rendered on the left.
fn compute_form(
    team_id: &str,
    matches: &[MatchDetails],
    max_results: usize,
    season_match_count: usize,
) -> Vec<FormEntry> {
    let mut finished: Vec<&MatchDetails> = matches
        .iter()
        .filter(|m| {
            m.Status.to_uppercase() == "FINISHED"
                && m.MatchType == 1 // league only
                && (m.HomeTeam.HomeTeamID == team_id || m.AwayTeam.AwayTeamID == team_id)
                && m.HomeGoals.is_some()
                && m.AwayGoals.is_some()
        })
        .collect();

    // Sort descending so we can take the most-recent N (current season) first.
    finished.sort_by(|a, b| b.MatchDate.cmp(&a.MatchDate));

    let limit = max_results.min(season_match_count);

    finished
        .into_iter()
        .take(limit)
        .filter_map(|m| {
            let home = m.HomeGoals?;
            let away = m.AwayGoals?;
            let is_home = m.HomeTeam.HomeTeamID == team_id;
            let outcome = if is_home {
                if home > away {
                    MatchOutcome::Win
                } else if home == away {
                    MatchOutcome::Draw
                } else {
                    MatchOutcome::Loss
                }
            } else if away > home {
                MatchOutcome::Win
            } else if away == home {
                MatchOutcome::Draw
            } else {
                MatchOutcome::Loss
            };
            Some(FormEntry {
                outcome,
                match_date: m.MatchDate.clone(),
                home_team: m.HomeTeam.HomeTeamName.clone(),
                away_team: m.AwayTeam.AwayTeamName.clone(),
                home_goals: home,
                away_goals: away,
            })
        })
        .collect()
}

// ── Badge colour from team ID ─────────────────────────────────────────────────

/// Maps a `team_id` string to a pastel HSL colour for the letter badge.
/// Uses a simple hash so each team gets a stable, distinct hue.
fn badge_colour(team_id: &str) -> (f64, f64, f64) {
    let hash: u64 = team_id.bytes().fold(5381u64, |acc, b| {
        acc.wrapping_mul(33).wrapping_add(b as u64)
    });
    let hue = (hash % 360) as f64 / 360.0;
    // Convert HSL(hue, 0.55, 0.55) → RGB
    hsl_to_rgb(hue, 0.55, 0.55)
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (f64, f64, f64) {
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;
    (
        hue_to_rgb(p, q, h + 1.0 / 3.0),
        hue_to_rgb(p, q, h),
        hue_to_rgb(p, q, h - 1.0 / 3.0),
    )
}

fn hue_to_rgb(p: f64, q: f64, mut t: f64) -> f64 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }
    if t < 1.0 / 6.0 {
        return p + (q - p) * 6.0 * t;
    }
    if t < 1.0 / 2.0 {
        return q;
    }
    if t < 2.0 / 3.0 {
        return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
    }
    p
}

// ── GObject wrapper for a league team row ────────────────────────────────────

mod imp_model {
    use super::*;
    use std::cell::RefCell;

    pub struct LeagueTeamObject {
        pub data: RefCell<Option<LeagueTeam>>,
        /// Up to 5 current-season league results, newest-first.
        pub form: RefCell<Vec<FormEntry>>,
        pub logo_url: RefCell<Option<String>>,
    }

    impl Default for LeagueTeamObject {
        fn default() -> Self {
            Self {
                data: RefCell::new(None),
                form: RefCell::new(Vec::new()),
                logo_url: RefCell::new(None),
            }
        }
    }

    #[glib::object_subclass]
    impl ObjectSubclass for LeagueTeamObject {
        const NAME: &'static str = "LeagueTeamObject";
        type Type = super::LeagueTeamObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for LeagueTeamObject {}
}

glib::wrapper! {
    pub struct LeagueTeamObject(ObjectSubclass<imp_model::LeagueTeamObject>);
}

impl LeagueTeamObject {
    pub fn new(team: LeagueTeam, form: Vec<FormEntry>, logo_url: Option<String>) -> Self {
        let obj: Self = glib::Object::builder().build();
        *obj.imp().data.borrow_mut() = Some(team);
        *obj.imp().form.borrow_mut() = form;
        *obj.imp().logo_url.borrow_mut() = logo_url;
        obj
    }
}

// ── GObject wrapper for a match row ─────────────────────────────────────────

mod imp_match {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    pub struct MatchObject {
        pub data: RefCell<Option<MatchDetails>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for MatchObject {
        const NAME: &'static str = "MatchObject";
        type Type = super::MatchObject;
        type ParentType = glib::Object;
    }

    impl ObjectImpl for MatchObject {}
}

glib::wrapper! {
    pub struct MatchObject(ObjectSubclass<imp_match::MatchObject>);
}

impl MatchObject {
    pub fn new(match_details: MatchDetails) -> Self {
        let obj: Self = glib::Object::builder().build();
        *obj.imp().data.borrow_mut() = Some(match_details);
        obj
    }
}

// ── SeriesPage widget ─────────────────────────────────────────────────────────

mod imp {
    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/Nutmeg/series/ui/page.ui")]
    pub struct SeriesPage {
        #[template_child]
        pub league_name_label: TemplateChild<Label>,
        #[template_child]
        pub league_table_view: TemplateChild<ColumnView>,
        #[template_child]
        pub matches_list_view: TemplateChild<ColumnView>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SeriesPage {
        const NAME: &'static str = "SeriesPage";
        type Type = super::SeriesPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for SeriesPage {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();
            obj.setup_columns();
        }
    }
    impl WidgetImpl for SeriesPage {}
    impl BoxImpl for SeriesPage {}
}

glib::wrapper! {
    pub struct SeriesPage(ObjectSubclass<imp::SeriesPage>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl Default for SeriesPage {
    fn default() -> Self {
        Self::new()
    }
}

impl SeriesPage {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn set_data(
        &self,
        league: Option<&LeagueDetailsData>,
        matches: Option<&MatchesData>,
        all_series_matches: &[MatchDetails],
        logo_urls: &std::collections::HashMap<i32, String>,
    ) {
        let imp = self.imp();

        // Use the all_series_matches dataset to compute forms so we have data for all teams
        let match_dataset = if all_series_matches.is_empty() {
            // Fallback just in case
            matches
                .map(|m| &m.Team.MatchList.Matches[..])
                .unwrap_or_default()
        } else {
            all_series_matches
        };

        if let Some(league_data) = league {
            debug!(
                "Setting league data for unit: {}",
                league_data.LeagueLevelUnitName
            );
            imp.league_name_label.set_text(&format!(
                "{} ({})",
                league_data.LeagueLevelUnitName, league_data.LeagueLevelUnitID
            ));

            let store = gtk::gio::ListStore::new::<LeagueTeamObject>();
            for team in &league_data.Teams {
                let form = compute_form(&team.TeamID, match_dataset, 5, team.Matches as usize);
                let tid = team.TeamID.parse::<i32>().unwrap_or(0);
                let logo_url = logo_urls.get(&tid).cloned();
                store.append(&LeagueTeamObject::new(team.clone(), form, logo_url));
            }
            debug!("Added {} teams to league store", league_data.Teams.len());
            let selection_model = gtk::NoSelection::new(Some(store));
            imp.league_table_view.set_model(Some(&selection_model));
        } else {
            debug!("No league data provided");
            imp.league_name_label.set_text("Series");
            imp.league_table_view.set_model(None::<&gtk::NoSelection>);
        }

        if let Some(matches_data) = matches {
            let store = gtk::gio::ListStore::new::<MatchObject>();
            for match_details in &matches_data.Team.MatchList.Matches {
                store.append(&MatchObject::new(match_details.clone()));
            }
            let selection_model = gtk::NoSelection::new(Some(store));
            imp.matches_list_view.set_model(Some(&selection_model));
        } else {
            imp.matches_list_view.set_model(None::<&gtk::NoSelection>);
        }
    }

    fn setup_columns(&self) {
        let imp = self.imp();

        // ── Series Table ──────────────────────────────────────────────────────
        let view = &imp.league_table_view;

        // translators: league table column headers (keep abbreviations short)
        self.add_league_text_column(view, &gettext("Pos"), |t| t.Position.to_string());
        self.add_badge_name_column(view);
        self.add_league_text_column(view, &gettext("P"), |t| t.Matches.to_string());
        self.add_league_text_column(view, &gettext("W"), |t| t.Won.to_string());
        self.add_league_text_column(view, &gettext("D"), |t| t.Draws.to_string());
        self.add_league_text_column(view, &gettext("L"), |t| t.Lost.to_string());
        self.add_league_text_column(view, &gettext("GD"), |t| {
            (t.GoalsFor as i32 - t.GoalsAgainst as i32).to_string()
        });
        self.add_league_text_column(view, &gettext("Pts"), |t| t.Points.to_string());
        self.add_form_column(view);

        // ── Matches ───────────────────────────────────────────────────────────
        let matches_view = &imp.matches_list_view;

        self.add_match_column(matches_view, &gettext("Date"), false, |m| m.MatchDate.clone());
        self.add_match_column(matches_view, &gettext("Home"), true, |m| {
            m.HomeTeam.HomeTeamName.clone()
        });
        self.add_match_column(matches_view, &gettext("Score"), false, format_match_score);
        self.add_match_column(matches_view, &gettext("Away"), true, |m| {
            m.AwayTeam.AwayTeamName.clone()
        });
    }

    // ── Column builders ───────────────────────────────────────────────────────

    /// Simple text column for league table rows.
    fn add_league_text_column<F>(&self, view: &ColumnView, title: &str, extractor: F)
    where
        F: Fn(&LeagueTeam) -> String + 'static + Clone,
    {
        let factory = SignalListItemFactory::new();
        let ext = extractor.clone();

        factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<ListItem>().unwrap();
            let label = Label::new(None);
            label.set_halign(gtk::Align::Start);
            label.set_valign(gtk::Align::Center);
            label.set_margin_top(8);
            label.set_margin_bottom(8);
            item.set_child(Some(&label));
        });

        factory.connect_bind(move |_, item| {
            let item = item.downcast_ref::<ListItem>().unwrap();
            if let Some(obj) = item.item().and_downcast::<LeagueTeamObject>() {
                if let Some(label) = item.child().and_downcast::<Label>() {
                    let text = obj
                        .imp()
                        .data
                        .borrow()
                        .as_ref()
                        .map(|t| ext(t))
                        .unwrap_or_default();
                    label.set_text(&text);
                }
            }
        });

        let column = ColumnViewColumn::new(Some(title), Some(factory));
        view.append_column(&column);
    }

    /// Team column: coloured letter-badge disc + team name label side-by-side.
    fn add_badge_name_column(&self, view: &ColumnView) {
        let factory = SignalListItemFactory::new();

        factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<ListItem>().unwrap();

            let row = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            row.set_margin_top(6);
            row.set_margin_bottom(6);

            // Fallback: coloured letter-badge disc drawn with Cairo.
            let badge = DrawingArea::new();
            badge.set_content_width(28);
            badge.set_content_height(28);
            badge.set_valign(gtk::Align::Center);

            // Preferred: the actual team logo image.
            let badge_img = gtk::Image::new();
            badge_img.set_pixel_size(28);
            badge_img.set_valign(gtk::Align::Center);
            badge_img.set_visible(false);

            let name_label = Label::new(None);
            name_label.set_halign(gtk::Align::Start);
            name_label.set_valign(gtk::Align::Center);

            row.append(&badge);
            row.append(&badge_img);
            row.append(&name_label);
            item.set_child(Some(&row));
        });

        thread_local! {
            static IMAGE_CACHE: std::cell::RefCell<std::collections::HashMap<String, gtk::gdk::Texture>> = Default::default();
        }

        factory.connect_bind(move |_, item| {
            let item = item.downcast_ref::<ListItem>().unwrap();
            if let Some(obj) = item.item().and_downcast::<LeagueTeamObject>() {
                let data = obj.imp().data.borrow();
                let logo_url = obj.imp().logo_url.borrow().clone();
                if let Some(team) = data.as_ref() {
                    let team_name = team.TeamName.clone();
                    let team_id = team.TeamID.clone();

                    if let Some(row) = item.child().and_downcast::<gtk::Box>() {
                        let badge_draw = row.first_child().and_downcast::<DrawingArea>().unwrap();
                        let badge_img = badge_draw
                            .next_sibling()
                            .and_downcast::<gtk::Image>()
                            .unwrap();
                        let name_label = badge_img.next_sibling().and_downcast::<Label>().unwrap();

                        name_label.set_text(&team_name);
                        badge_img.set_widget_name(&team_id); // Tag image widget with team_id for async validation

                        // Update fallback badge draw function
                        let first_char = team_name
                            .chars()
                            .next()
                            .and_then(|c| c.to_uppercase().next())
                            .unwrap_or('?');
                        let (br, bg, bb) = badge_colour(&team_id);
                        badge_draw.set_draw_func(move |_, cr, w, h| {
                            let cx = w as f64 / 2.0;
                            let cy = h as f64 / 2.0;
                            let r = (w.min(h) as f64 / 2.0) - 0.5;

                            // Filled circle
                            cr.arc(cx, cy, r, 0.0, std::f64::consts::TAU);
                            cr.set_source_rgb(br, bg, bb);
                            let _ = cr.fill();

                            // White initial letter
                            cr.set_source_rgb(1.0, 1.0, 1.0);
                            cr.select_font_face(
                                "Sans",
                                gtk::cairo::FontSlant::Normal,
                                gtk::cairo::FontWeight::Bold,
                            );
                            cr.set_font_size(10.0);
                            let letter = first_char.to_string();
                            if let Ok(ext) = cr.text_extents(&letter) {
                                cr.move_to(
                                    cx - ext.width() / 2.0 - ext.x_bearing(),
                                    cy - ext.height() / 2.0 - ext.y_bearing(),
                                );
                                let _ = cr.show_text(&letter);
                            }
                        });

                        if let Some(url_str) = logo_url {
                            let fixed_url = if url_str.starts_with("//") {
                                format!("https:{}", url_str)
                            } else {
                                url_str
                            };

                            // Serve from in-memory cache when available.
                            let cached_texture =
                                IMAGE_CACHE.with(|cache| cache.borrow().get(&fixed_url).cloned());
                            if let Some(texture) = cached_texture {
                                badge_draw.set_visible(false);
                                badge_img.set_visible(true);
                                badge_img.set_paintable(Some(&texture));
                            } else {
                                // Show the letter-badge fallback while the logo is loading.
                                badge_draw.set_visible(true);
                                badge_img.set_visible(false);
                                badge_img.set_paintable(None::<&gtk::gdk::Texture>);

                                let img_weak = badge_img.downgrade();
                                let draw_weak = badge_draw.downgrade();
                                let target_id = team_id.clone();
                                glib::MainContext::default().spawn_local(async move {
                                    // Another concurrent task may have already populated the cache.
                                    if let Some(texture) =
                                        IMAGE_CACHE.with(|c| c.borrow().get(&fixed_url).cloned())
                                    {
                                        if let Some(img) = img_weak.upgrade() {
                                            if img.widget_name() == target_id {
                                                img.set_paintable(Some(&texture));
                                                img.set_visible(true);
                                                if let Some(draw) = draw_weak.upgrade() {
                                                    draw.set_visible(false);
                                                }
                                            }
                                        }
                                        return;
                                    }

                                    match crate::utils::image::load_image_from_url(&fixed_url).await
                                    {
                                        Ok(texture) => {
                                            IMAGE_CACHE.with(|c| {
                                                c.borrow_mut()
                                                    .insert(fixed_url.clone(), texture.clone());
                                            });
                                            if let Some(img) = img_weak.upgrade() {
                                                // Guard against the row being recycled for a
                                                // different team before the download finished.
                                                if img.widget_name() == target_id {
                                                    img.set_paintable(Some(&texture));
                                                    img.set_visible(true);
                                                    if let Some(draw) = draw_weak.upgrade() {
                                                        draw.set_visible(false);
                                                    }
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            warn!(
                                                "Failed to load team logo from '{}': {}",
                                                fixed_url, e
                                            );
                                        }
                                    }
                                });
                            }
                        } else {
                            badge_img.set_visible(false);
                            badge_draw.set_visible(true);
                        }
                    }
                }
            }
        });

        let column = ColumnViewColumn::new(Some(&gettext("Team")), Some(factory));
        column.set_expand(true);
        view.append_column(&column);
    }

    /// Form column: up to 5 coloured discs showing the current season's most-recent
    /// results, newest on the left.  Discs beyond the number of games played this
    /// season are hidden so the column is empty at the start of a new season.
    /// Hovering a disc shows a tooltip with the match date and score.
    fn add_form_column(&self, view: &ColumnView) {
        const MAX_DISCS: usize = 5;
        const DISC_SIZE: i32 = 14;

        let factory = SignalListItemFactory::new();

        factory.connect_setup(move |_, item| {
            let item = item.downcast_ref::<ListItem>().unwrap();
            let row = gtk::Box::new(gtk::Orientation::Horizontal, 3);
            row.set_margin_top(8);
            row.set_margin_bottom(8);
            row.set_valign(gtk::Align::Center);
            for _ in 0..MAX_DISCS {
                let disc = DrawingArea::new();
                disc.set_content_width(DISC_SIZE);
                disc.set_content_height(DISC_SIZE);
                disc.set_valign(gtk::Align::Center);
                // Enable tooltips on the drawing area.
                disc.set_has_tooltip(true);
                row.append(&disc);
            }
            item.set_child(Some(&row));
        });

        factory.connect_bind(move |_, item| {
            let item = item.downcast_ref::<ListItem>().unwrap();
            if let Some(obj) = item.item().and_downcast::<LeagueTeamObject>() {
                let form = obj.imp().form.borrow().clone();
                // `form` is newest-first; index 0 → leftmost disc.
                if let Some(row) = item.child().and_downcast::<gtk::Box>() {
                    let mut child_opt = row.first_child();
                    for i in 0..MAX_DISCS {
                        if let Some(child) = child_opt.take() {
                            if let Some(disc) = child.downcast_ref::<DrawingArea>() {
                                if let Some(entry) = form.get(i) {
                                    let colour = entry.outcome.colour();
                                    disc.set_visible(true);
                                    disc.set_draw_func(move |_, cr, w, h| {
                                        let cx = w as f64 / 2.0;
                                        let cy = h as f64 / 2.0;
                                        let r = (w.min(h) as f64 / 2.0) - 1.0;
                                        cr.arc(cx, cy, r, 0.0, std::f64::consts::TAU);
                                        let (dr, dg, db) = colour;
                                        cr.set_source_rgb(dr, dg, db);
                                        let _ = cr.fill();
                                    });
                                    // translators: Form disc tooltip.
                                    // {date} = match date, {home} = home team name,
                                    // {hg} = home goals, {ag} = away goals,
                                    // {away} = away team name.
                                    let tooltip = gettext("{date}\n{home} {hg} \u{2013} {ag} {away}")
                                        .replace("{date}", &entry.match_date)
                                        .replace("{home}", &entry.home_team)
                                        .replace("{hg}", &entry.home_goals.to_string())
                                        .replace("{ag}", &entry.away_goals.to_string())
                                        .replace("{away}", &entry.away_team);
                                    disc.set_tooltip_text(Some(&tooltip));
                                } else {
                                    // No game for this slot in the current season.
                                    disc.set_visible(false);
                                    disc.set_tooltip_text(None);
                                    disc.set_draw_func(|_, _, _, _| {});
                                }
                            }
                            child_opt = child.next_sibling();
                        }
                    }
                }
            }
        });

        let column = ColumnViewColumn::new(Some("Form"), Some(factory));
        view.append_column(&column);
    }

    /// Simple text column for match rows.
    fn add_match_column<F>(&self, view: &ColumnView, title: &str, expand: bool, extractor: F)
    where
        F: Fn(&MatchDetails) -> String + 'static + Clone,
    {
        let factory = SignalListItemFactory::new();
        let ext = extractor.clone();

        factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<ListItem>().unwrap();
            let label = Label::new(None);
            label.set_halign(gtk::Align::Start);
            label.set_valign(gtk::Align::Center);
            label.set_margin_top(8);
            label.set_margin_bottom(8);
            item.set_child(Some(&label));
        });

        factory.connect_bind(move |_, item| {
            let item = item.downcast_ref::<ListItem>().unwrap();
            if let Some(obj) = item.item().and_downcast::<MatchObject>() {
                if let Some(label) = item.child().and_downcast::<Label>() {
                    let text = obj
                        .imp()
                        .data
                        .borrow()
                        .as_ref()
                        .map(|m| ext(m))
                        .unwrap_or_default();
                    label.set_text(&text);
                }
            }
        });

        let column = ColumnViewColumn::new(Some(title), Some(factory));
        if expand {
            column.set_expand(true);
        }
        view.append_column(&column);
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn format_match_score(details: &MatchDetails) -> String {
    match (details.HomeGoals, details.AwayGoals) {
        // translators: Football match score. {home} = home goals, {away} = away goals.
        // The separator and order can be changed for regional conventions (e.g. "3 : 2").
        (Some(h), Some(a)) => gettext("{home} \u{2013} {away}")
            .replace("{home}", &h.to_string())
            .replace("{away}", &a.to_string()),
        _ => "-".to_string(),
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chpp::model::{MatchAwayTeam, MatchHomeTeam};

    fn make_match(
        match_id: u32,
        home_id: &str,
        away_id: &str,
        home_goals: u32,
        away_goals: u32,
        date: &str,
    ) -> MatchDetails {
        MatchDetails {
            MatchID: match_id,
            HomeTeam: MatchHomeTeam {
                HomeTeamID: home_id.to_string(),
                HomeTeamName: format!("Team {}", home_id),
                ..Default::default()
            },
            AwayTeam: MatchAwayTeam {
                AwayTeamID: away_id.to_string(),
                AwayTeamName: format!("Team {}", away_id),
                ..Default::default()
            },
            MatchDate: date.to_string(),
            MatchType: 1,
            Status: "FINISHED".to_string(),
            HomeGoals: Some(home_goals),
            AwayGoals: Some(away_goals),
            ..Default::default()
        }
    }

    fn outcomes(form: &[FormEntry]) -> Vec<MatchOutcome> {
        form.iter().map(|e| e.outcome).collect()
    }

    #[test]
    fn test_compute_form_win_draw_loss_newest_first() {
        // Chronological order: Win → Draw → Loss; newest-first expected: Loss, Draw, Win.
        let matches = vec![
            make_match(1, "10", "20", 2, 0, "2026-01-01 20:00:00"), // team 10 wins
            make_match(2, "20", "10", 1, 1, "2026-01-08 20:00:00"), // draw
            make_match(3, "10", "30", 0, 2, "2026-01-15 20:00:00"), // team 10 loses
        ];
        let form = compute_form("10", &matches, 5, 3);
        assert_eq!(
            outcomes(&form),
            vec![MatchOutcome::Loss, MatchOutcome::Draw, MatchOutcome::Win]
        );
    }

    #[test]
    fn test_compute_form_only_last_five() {
        // 8 matches played this season; form bar capped at 5 (the 5 most recent).
        let matches: Vec<MatchDetails> = (1..=8)
            .map(|i| make_match(i, "10", "20", 1, 0, &format!("2026-01-{:02} 20:00:00", i)))
            .collect();
        let form = compute_form("10", &matches, 5, 8);
        assert_eq!(form.len(), 5);
        assert!(form.iter().all(|e| e.outcome == MatchOutcome::Win));
    }

    #[test]
    fn test_compute_form_scoped_to_current_season() {
        // 5 matches in the dataset but only 2 played in the current season.
        // The form bar must show 2 discs (the 2 most recent matches).
        let matches: Vec<MatchDetails> = (1..=5)
            .map(|i| make_match(i, "10", "20", 1, 0, &format!("2026-01-{:02} 20:00:00", i)))
            .collect();
        let form = compute_form("10", &matches, 5, 2);
        assert_eq!(form.len(), 2);
    }

    #[test]
    fn test_compute_form_new_season_is_empty() {
        // At the start of a new season, season_match_count == 0 → column empty.
        let matches: Vec<MatchDetails> = (1..=5)
            .map(|i| make_match(i, "10", "20", 1, 0, &format!("2026-01-{:02} 20:00:00", i)))
            .collect();
        let form = compute_form("10", &matches, 5, 0);
        assert!(form.is_empty());
    }

    #[test]
    fn test_compute_form_excludes_cup_matches() {
        let mut m = make_match(1, "10", "20", 3, 1, "2026-01-01 20:00:00");
        m.MatchType = 3; // cup — must be excluded
        let form = compute_form("10", &[m], 5, 1);
        assert!(form.is_empty());
    }

    #[test]
    fn test_compute_form_tooltip_fields() {
        let m = make_match(1, "10", "20", 2, 1, "2026-03-01 20:00:00");
        let form = compute_form("10", &[m], 5, 1);
        assert_eq!(form.len(), 1);
        let entry = &form[0];
        assert_eq!(entry.outcome, MatchOutcome::Win);
        assert_eq!(entry.home_goals, 2);
        assert_eq!(entry.away_goals, 1);
        assert_eq!(entry.match_date, "2026-03-01 20:00:00");
        assert_eq!(entry.home_team, "Team 10");
        assert_eq!(entry.away_team, "Team 20");
    }

    // ── Regression tests for the two fixed bugs ──────────────────────────────

    #[test]
    fn test_compute_form_includes_match_with_context_id_zero() {
        let mut m = make_match(1, "10", "20", 2, 1, "2026-01-01 20:00:00");
        m.MatchContextId = Some(0);
        let form = compute_form("10", &[m], 5, 1);
        assert_eq!(outcomes(&form), vec![MatchOutcome::Win]);
    }

    #[test]
    fn test_compute_form_includes_match_with_context_id_none() {
        let mut m = make_match(1, "10", "20", 1, 2, "2026-01-01 20:00:00");
        m.MatchContextId = None;
        let form = compute_form("10", &[m], 5, 1);
        assert_eq!(outcomes(&form), vec![MatchOutcome::Loss]);
    }

    #[test]
    fn test_compute_form_not_affected_by_context_id_value() {
        let league_unit_id = 54321u32;
        let mut with_correct = make_match(1, "10", "20", 2, 0, "2026-01-01 20:00:00");
        with_correct.MatchContextId = Some(league_unit_id);

        let mut with_zero = make_match(2, "10", "30", 1, 1, "2026-01-08 20:00:00");
        with_zero.MatchContextId = Some(0);

        let mut with_none = make_match(3, "10", "40", 0, 1, "2026-01-15 20:00:00");
        with_none.MatchContextId = None;

        // Newest-first: Loss (match 3) → Draw (match 2) → Win (match 1).
        let form = compute_form("10", &[with_correct, with_zero, with_none], 5, 3);
        assert_eq!(
            outcomes(&form),
            vec![MatchOutcome::Loss, MatchOutcome::Draw, MatchOutcome::Win]
        );
    }

    #[test]
    fn test_badge_colour_is_stable() {
        let (r1, g1, b1) = badge_colour("12345");
        let (r2, g2, b2) = badge_colour("12345");
        assert!((r1 - r2).abs() < f64::EPSILON);
        assert!((g1 - g2).abs() < f64::EPSILON);
        assert!((b1 - b2).abs() < f64::EPSILON);
    }
}
