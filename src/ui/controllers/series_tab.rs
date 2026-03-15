use crate::ui::context_object::ContextObject;

pub struct SeriesTabController {
    context: ContextObject,
}

impl SeriesTabController {
    pub fn new(context: ContextObject) -> Self {
        Self { context }
    }

    /// Loads series data for the selected team eagerly from the local DB.
    ///
    /// All reads are synchronous — SQLite lookups are fast enough to run on the GTK
    /// main thread without perceptible lag. Only the logo image download (URL → pixel
    /// data) remains asynchronous; that happens later in the column-bind callback of
    /// `SeriesPage::add_badge_name_column`.
    ///
    /// All four series context fields are written atomically via `set_series_data` so
    /// the series page redraws exactly once.
    pub fn clear(&self) {
        // No-op: ContextObject clears its own properties.
    }
}
