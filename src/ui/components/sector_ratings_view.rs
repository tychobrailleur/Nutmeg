use gettextrs::gettext;
use gtk::prelude::*;

/// A specific layout for displaying sector ratings (e.g., in Opponent Analysis).
/// It renders Left Attack, Centre Attack, Right Attack at the top;
/// Midfield in the middle; and Left Defence, Centre Defence, Right Defence at the bottom.
pub struct SectorRatingsView;

impl SectorRatingsView {
    /// Creates the pitch visualization specifically for sector ratings.
    /// Expects 7 ratings in the order: [LAtt, CAtt, RAtt, Mid, LDef, CDef, RDef].
    pub fn create(ratings: &[f64; 7]) -> gtk::Grid {
        let grid = gtk::Grid::builder()
            .row_spacing(4)
            .column_spacing(2)
            .column_homogeneous(true)
            .row_homogeneous(true)
            .vexpand(true)
            .build();
        grid.add_css_class("pitch-view");

        let slot_size_group = gtk::SizeGroup::new(gtk::SizeGroupMode::Both);

        let create_slot = |label_text: &str, rating: f64| -> gtk::Widget {
            let container = gtk::Box::new(gtk::Orientation::Vertical, 0);
            container.set_halign(gtk::Align::Fill);
            container.set_valign(gtk::Align::Center);

            let slot_label = gtk::Label::builder()
                .label(label_text)
                .css_classes(["caption", "dim-label"])
                .halign(gtk::Align::Center)
                .build();
            container.append(&slot_label);

            let rating_label = gtk::Label::builder()
                .label(&format!("{:.1}", rating))
                .ellipsize(gtk::pango::EllipsizeMode::End)
                .max_width_chars(10)
                .css_classes(["body", "strong"])
                .halign(gtk::Align::Center)
                .build();

            container.append(&rating_label);
            container.add_css_class("slot-filled");

            // Add tooltip with long translated names
            let tooltip = format!("{} Rating: {:.1}", label_text, rating);
            container.set_tooltip_text(Some(&tooltip));

            container.upcast::<gtk::Widget>()
        };

        // --- Row 0: Attack (Left, Centre, Right) ---
        grid.attach(
            &create_slot(&gettext("Left Attack"), ratings[0]),
            0,
            0,
            1,
            1,
        );
        grid.attach(
            &create_slot(&gettext("Centre Attack"), ratings[1]),
            2,
            0,
            1,
            1,
        );
        grid.attach(
            &create_slot(&gettext("Right Attack"), ratings[2]),
            4,
            0,
            1,
            1,
        );

        // --- Row 1: Midfield ---
        // Center the midfield rating
        grid.attach(&create_slot(&gettext("Midfield"), ratings[3]), 1, 1, 3, 1);

        // --- Row 2: Defence (Left, Centre, Right) ---
        grid.attach(
            &create_slot(&gettext("Left Defence"), ratings[4]),
            0,
            2,
            1,
            1,
        );
        grid.attach(
            &create_slot(&gettext("Centre Defence"), ratings[5]),
            2,
            2,
            1,
            1,
        );
        grid.attach(
            &create_slot(&gettext("Right Defence"), ratings[6]),
            4,
            2,
            1,
            1,
        );

        grid
    }
}
