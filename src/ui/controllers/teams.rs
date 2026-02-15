use gtk::prelude::*;
use gtk::{gio, glib};
use log::{error, info, debug};
use crate::db::manager::DbManager;
use crate::db::teams::get_teams_summary;
use crate::ui::team_object::TeamObject;
use crate::utils::image::load_image_from_url;

pub struct TeamController;

impl TeamController {
    
    pub fn load_teams(dropdown: &gtk::DropDown) {
        let db = DbManager::new();
        if let Ok(mut conn) = db.get_connection() {
            match get_teams_summary(&mut conn) {
                Ok(teams) => {
                    info!("Loaded {} teams", teams.len());

                    let model = gio::ListStore::new::<TeamObject>();
                    for (id, name, logo_url) in teams {
                        model.append(&TeamObject::new(id, name, logo_url));
                    }

                    // Setup factory
                    let factory = gtk::SignalListItemFactory::new();
                    Self::setup_factory(&factory);

                    dropdown.set_model(Some(&model));
                    dropdown.set_factory(Some(&factory));

                    if model.n_items() > 0 {
                        dropdown.set_selected(0);
                    }
                }
                Err(e) => error!("Failed to load teams: {}", e),
            }
        } else {
            error!("Failed to get DB connection");
        }
    }

    fn setup_factory(factory: &gtk::SignalListItemFactory) {
        factory.connect_setup(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 8);
            hbox.set_margin_start(4);
            hbox.set_margin_end(4);
            hbox.set_margin_top(4);
            hbox.set_margin_bottom(4);

            let logo = gtk::Image::new();
            logo.set_pixel_size(32);
            hbox.append(&logo);

            let label = gtk::Label::new(None);
            label.set_xalign(0.0);
            hbox.append(&label);

            item.set_child(Some(&hbox));
        });

        factory.connect_bind(|_, item| {
            let item = item.downcast_ref::<gtk::ListItem>().unwrap();
            let team_obj = item.item().and_downcast::<TeamObject>().unwrap();
            let hbox = item.child().and_downcast::<gtk::Box>().unwrap();

            let logo = hbox.first_child().unwrap().downcast::<gtk::Image>().unwrap();
            let label = logo.next_sibling().unwrap().downcast::<gtk::Label>().unwrap();

            let team_data = team_obj.team_data();

            let markup = format!(
                "{} <span foreground='gray'>({})</span>",
                glib::markup_escape_text(&team_data.name),
                team_data.id
            );
            label.set_markup(&markup);

             if let Some(mut url) = team_data.logo_url {
                if url.starts_with("//") {
                    url = format!("https:{}", url);
                }

                let logo_clone = logo.clone();
                glib::MainContext::default().spawn_local(async move {
                    match load_image_from_url(&url).await {
                        Ok(texture) => {
                            logo_clone.set_paintable(Some(&texture));
                        }
                        Err(e) => {
                            debug!("Failed to load team logo from {}: {}", url, e);
                        }
                    }
                });
            }
        });
    }
}
