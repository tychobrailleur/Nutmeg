/* series_page.rs
 *
 * Copyright 2026 Sébastien Le Callonnec
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use crate::chpp::model::{LeagueDetailsData, LeagueTeam, MatchDetails, MatchesData};
use glib::subclass::InitializingObject;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{
    glib, ColumnView, ColumnViewColumn, CompositeTemplate, Label, ListItem, SignalListItemFactory,
};

// Object to wrap LeagueTeam for GListStore
mod imp_model {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    pub struct LeagueTeamObject {
        pub data: RefCell<Option<LeagueTeam>>,
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
    pub fn new(team: LeagueTeam) -> Self {
        let obj: Self = glib::Object::builder().build();
        *obj.imp().data.borrow_mut() = Some(team);
        obj
    }
}

// Object to wrap MatchDetails for GListStore
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

mod imp {
    use super::*;

    #[derive(Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/Nutmeg/series/page.ui")]
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

    pub fn set_data(&self, league: Option<&LeagueDetailsData>, matches: Option<&MatchesData>) {
        let imp = self.imp();

        if let Some(league_data) = league {
            log::debug!(
                "Setting league data for unit: {}",
                league_data.LeagueLevelUnitName
            );
            imp.league_name_label.set_text(&format!(
                "{} ({})",
                league_data.LeagueLevelUnitName, league_data.LeagueLevelUnitID
            ));

            let store = gtk::gio::ListStore::new::<LeagueTeamObject>();
            for team in &league_data.Teams {
                store.append(&LeagueTeamObject::new(team.clone()));
            }
            log::debug!("Added {} teams to league store", league_data.Teams.len());
            let selection_model = gtk::NoSelection::new(Some(store));
            imp.league_table_view.set_model(Some(&selection_model));
        } else {
            log::debug!("No league data provided");
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

        // --- League Table ---
        let view = &imp.league_table_view;

        self.add_column(view, "Pos", |obj: &LeagueTeamObject| {
            obj.imp()
                .data
                .borrow()
                .as_ref()
                .map(|team| team.Position.to_string())
                .unwrap_or_default()
        });

        self.add_column(view, "Team", |obj: &LeagueTeamObject| {
            obj.imp()
                .data
                .borrow()
                .as_ref()
                .map(|team| team.TeamName.clone())
                .unwrap_or_default()
        });

        self.add_column(view, "P", |obj: &LeagueTeamObject| {
            obj.imp()
                .data
                .borrow()
                .as_ref()
                .map(|team| team.Matches.to_string())
                .unwrap_or_default()
        });

        self.add_column(view, "W", |obj: &LeagueTeamObject| {
            obj.imp()
                .data
                .borrow()
                .as_ref()
                .map(|team| team.Won.to_string())
                .unwrap_or_default()
        });

        self.add_column(view, "D", |obj: &LeagueTeamObject| {
            obj.imp()
                .data
                .borrow()
                .as_ref()
                .map(|team| team.Draws.to_string())
                .unwrap_or_default()
        });

        self.add_column(view, "L", |obj: &LeagueTeamObject| {
            obj.imp()
                .data
                .borrow()
                .as_ref()
                .map(|team| team.Lost.to_string())
                .unwrap_or_default()
        });

        self.add_column(view, "GF", |obj: &LeagueTeamObject| {
            obj.imp()
                .data
                .borrow()
                .as_ref()
                .map(|team| team.GoalsFor.to_string())
                .unwrap_or_default()
        });
        self.add_column(view, "GA", |obj: &LeagueTeamObject| {
            obj.imp()
                .data
                .borrow()
                .as_ref()
                .map(|team| team.GoalsAgainst.to_string())
                .unwrap_or_default()
        });
        self.add_column(view, "GD", |obj: &LeagueTeamObject| {
            obj.imp()
                .data
                .borrow()
                .as_ref()
                .map(calculate_goal_difference)
                .unwrap_or_default()
        });
        self.add_column(view, "Pts", |obj: &LeagueTeamObject| {
            obj.imp()
                .data
                .borrow()
                .as_ref()
                .map(|team| team.Points.to_string())
                .unwrap_or_default()
        });

        // --- Matches ---
        let matches_view = &imp.matches_list_view;

        self.add_match_column(matches_view, "Date", |obj: &MatchObject| {
            obj.imp()
                .data
                .borrow()
                .as_ref()
                .map(|details| details.MatchDate.clone())
                .unwrap_or_default()
        });
        self.add_match_column(matches_view, "Home", |obj: &MatchObject| {
            obj.imp()
                .data
                .borrow()
                .as_ref()
                .map(|details| details.HomeTeam.HomeTeamName.clone())
                .unwrap_or_default()
        });
        self.add_match_column(matches_view, "Score", |obj: &MatchObject| {
            obj.imp()
                .data
                .borrow()
                .as_ref()
                .map(format_match_score)
                .unwrap_or_default()
        });
        self.add_match_column(matches_view, "Away", |obj: &MatchObject| {
            obj.imp()
                .data
                .borrow()
                .as_ref()
                .map(|details| details.AwayTeam.AwayTeamName.clone())
                .unwrap_or_default()
        });
    }

    fn add_column<F>(&self, view: &ColumnView, title: &str, extractor: F)
    where
        F: Fn(&LeagueTeamObject) -> String + 'static + Clone,
    {
        let factory = SignalListItemFactory::new();
        let extractor_clone = extractor.clone();

        factory.connect_setup(move |_, item| {
            let item = item.downcast_ref::<ListItem>().unwrap();
            let label = Label::new(None);
            label.set_halign(gtk::Align::Start);
            item.set_child(Some(&label));
        });

        factory.connect_bind(move |_, item| {
            let item = item.downcast_ref::<ListItem>().unwrap();
            if let Some(obj) = item.item().and_downcast::<LeagueTeamObject>() {
                let label = item.child().and_downcast::<Label>().unwrap();
                let text = extractor_clone(&obj);
                label.set_text(&text);
            }
        });

        let column = ColumnViewColumn::new(Some(title), Some(factory));
        if title == "Team" {
            column.set_expand(true);
        }
        view.append_column(&column);
    }

    fn add_match_column<F>(&self, view: &ColumnView, title: &str, extractor: F)
    where
        F: Fn(&MatchObject) -> String + 'static + Clone,
    {
        let factory = SignalListItemFactory::new();
        let extractor_clone = extractor.clone();

        factory.connect_setup(move |_, item| {
            let item = item.downcast_ref::<ListItem>().unwrap();
            let label = Label::new(None);
            label.set_halign(gtk::Align::Start);
            item.set_child(Some(&label));
        });

        factory.connect_bind(move |_, item| {
            let item = item.downcast_ref::<ListItem>().unwrap();
            if let Some(obj) = item.item().and_downcast::<MatchObject>() {
                let label = item.child().and_downcast::<Label>().unwrap();
                let text = extractor_clone(&obj);
                label.set_text(&text);
            }
        });

        let column = ColumnViewColumn::new(Some(title), Some(factory));
        if title == "Home" || title == "Away" {
            column.set_expand(true);
        }
        view.append_column(&column);
    }
}

fn calculate_goal_difference(team: &LeagueTeam) -> String {
    (team.GoalsFor as i32 - team.GoalsAgainst as i32).to_string()
}

fn format_match_score(details: &MatchDetails) -> String {
    if let (Some(home_goals), Some(away_goals)) = (details.HomeGoals, details.AwayGoals) {
        format!("{} - {}", home_goals, away_goals)
    } else {
        "-".to_string()
    }
}
