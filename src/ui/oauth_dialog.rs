use gtk::prelude::*;
use gtk::{Button, Entry, ResponseType};

pub struct OAuthDialog {
    dialog: gtk::Dialog,
    entry: Entry,
}

impl OAuthDialog {
    pub fn new(parent: &impl IsA<gtk::Window>) -> Self {
        let dialog = gtk::Dialog::builder()
            .transient_for(parent)
            .modal(true)
            .title("Hattrick Authorization")
            .width_request(400)
            .build();

        let content_area = dialog.content_area();
        content_area.set_spacing(12);
        content_area.set_margin_top(12);
        content_area.set_margin_bottom(12);
        content_area.set_margin_start(12);
        content_area.set_margin_end(12);

        let label = gtk::Label::builder()
            .label("Authorization required.\nPlease authorize Nutmeg in the opened browser window\nand paste the verification code below:")
            .wrap(true)
            .xalign(0.0)
            .build();
        content_area.append(&label);

        let entry = Entry::builder()
            .placeholder_text("Verification Code")
            .activates_default(true)
            .build();
        content_area.append(&entry);

        // Add buttons
        dialog.add_button("Cancel", ResponseType::Cancel);
        dialog.add_button("Verify", ResponseType::Ok);
        dialog.set_default_response(ResponseType::Ok);

        Self { dialog, entry }
    }

    pub async fn run(&self) -> Option<String> {
        self.dialog.show();
        let response = self.dialog.run_future().await;

        let result = if response == ResponseType::Ok {
            let text = self.entry.text().to_string();
            if text.trim().is_empty() {
                None
            } else {
                Some(text)
            }
        } else {
            None
        };

        self.dialog.close();
        result
    }
}
