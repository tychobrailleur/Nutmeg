use crate::rating::types::{Behaviour, PositionId};
use gtk::prelude::*;

/// A generic representation of a player on the pitch
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct PitchPlayer {
    pub id: u32,
    pub first_name: String,
    pub last_name: String,
    pub role: PositionId,
    pub behaviour: Behaviour,
    /// An optional rating that can be displayed as a tooltip or secondary label
    pub rating: Option<f64>,
}

impl PitchPlayer {
    pub fn short_name(&self) -> String {
        format!(
            "{}. {}",
            self.first_name.chars().next().unwrap_or('?'),
            self.last_name
        )
    }
}

pub struct PitchView;

impl PitchView {
    /// Creates the pitch visualisation with all position slots.
    /// `players` should be a slice or Vec of PitchPlayer.
    pub fn create(players: &[PitchPlayer]) -> gtk::Grid {
        let grid = gtk::Grid::builder()
            .row_spacing(4)
            .column_spacing(2)
            .column_homogeneous(true)
            .row_homogeneous(true)
            .vexpand(true)
            .build();
        grid.add_css_class("pitch-view");

        let slot_size_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Both);

        let mut player_map = std::collections::HashMap::new();
        for p in players {
            player_map.insert(p.role, p);
        }

        let create_slot =
            |id: PositionId, label_text: &str, size_group: &gtk::SizeGroup| -> gtk::Widget {
                let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
                container.set_halign(gtk::Align::Fill);
                container.set_valign(gtk::Align::Center);

                let pos_row = gtk::Box::new(gtk::Orientation::Horizontal, 2);
                pos_row.set_halign(gtk::Align::Center);

                let slot_label = gtk::Label::builder()
                    .label(label_text)
                    .css_classes(["caption", "dim-label"])
                    .halign(gtk::Align::Center)
                    .build();
                pos_row.append(&slot_label);

                if let Some(player) = player_map.get(&id) {
                    let symbol = player.behaviour.symbol(&id);
                    if !symbol.is_empty() {
                        let triangle = gtk::Label::builder()
                            .label(symbol)
                            .css_classes(["caption", "behaviour-indicator"])
                            .halign(gtk::Align::Center)
                            .build();
                        pos_row.append(&triangle);
                    }
                }
                container.append(&pos_row);

                if let Some(player) = player_map.get(&id) {
                    let rating = player.rating.unwrap_or(0.0);
                    let tooltip = format!(
                        "{} {} ({} {}) - Rating: {:.1}",
                        player.first_name,
                        player.last_name,
                        label_text,
                        player.behaviour.name(),
                        rating
                    );

                    let name = gtk::Label::builder()
                        .label(&player.short_name())
                        .ellipsize(gtk::pango::EllipsizeMode::End)
                        .max_width_chars(10)
                        .css_classes(["body", "strong"])
                        .halign(gtk::Align::Center)
                        .build();

                    container.append(&name);
                    container.add_css_class("slot-filled");
                    container.set_tooltip_text(Some(&tooltip));
                } else {
                    Self::add_empty_slot_label(&container);
                }

                size_group.add_widget(&container);
                container.upcast()
            };

        // Row 0: Attack — columns 1, 2, 3
        grid.attach(
            &create_slot(PositionId::LeftForward, "LF", &slot_size_group),
            1,
            0,
            1,
            1,
        );
        grid.attach(
            &create_slot(PositionId::CentralForward, "CF", &slot_size_group),
            2,
            0,
            1,
            1,
        );
        grid.attach(
            &create_slot(PositionId::RightForward, "RF", &slot_size_group),
            3,
            0,
            1,
            1,
        );

        // Row 1: Midfield — all 5 columns
        grid.attach(
            &create_slot(PositionId::LeftWinger, "LW", &slot_size_group),
            0,
            1,
            1,
            1,
        );
        grid.attach(
            &create_slot(PositionId::LeftInnerMidfield, "LIM", &slot_size_group),
            1,
            1,
            1,
            1,
        );
        grid.attach(
            &create_slot(PositionId::CentralInnerMidfield, "MIM", &slot_size_group),
            2,
            1,
            1,
            1,
        );
        grid.attach(
            &create_slot(PositionId::RightInnerMidfield, "RIM", &slot_size_group),
            3,
            1,
            1,
            1,
        );
        grid.attach(
            &create_slot(PositionId::RightWinger, "RW", &slot_size_group),
            4,
            1,
            1,
            1,
        );

        // Row 2: Defence — all 5 columns
        grid.attach(
            &create_slot(PositionId::LeftBack, "LB", &slot_size_group),
            0,
            2,
            1,
            1,
        );
        grid.attach(
            &create_slot(PositionId::LeftCentralDefender, "LCD", &slot_size_group),
            1,
            2,
            1,
            1,
        );
        grid.attach(
            &create_slot(PositionId::MiddleCentralDefender, "MCD", &slot_size_group),
            2,
            2,
            1,
            1,
        );
        grid.attach(
            &create_slot(PositionId::RightCentralDefender, "RCD", &slot_size_group),
            3,
            2,
            1,
            1,
        );
        grid.attach(
            &create_slot(PositionId::RightBack, "RB", &slot_size_group),
            4,
            2,
            1,
            1,
        );

        // Row 3: Keeper — column 2
        grid.attach(
            &create_slot(PositionId::Keeper, "GK", &slot_size_group),
            2,
            3,
            1,
            1,
        );

        grid
    }

    fn add_empty_slot_label(container: &gtk::Box) {
        let empty = gtk::Label::builder()
            .label("-")
            .css_classes(["dim-label"])
            .halign(gtk::Align::Center)
            .build();
        container.append(&empty);
        container.add_css_class("slot-empty");
    }
}
