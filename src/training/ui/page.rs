#![allow(deprecated)] // GTK4 ListStore/TreeModel APIs deprecated since 4.10; migration is a separate effort

use crate::rating::PlayerSkill;
use crate::training::cycle::{CyclePlan, CycleStep};
use crate::training::service::TrainingService;
use gtk::prelude::*;
use gtk::subclass::prelude::*; // Import for ObjectSubclassIsExt (imp())
use gtk::{glib, CompositeTemplate};
use std::cell::RefCell;

mod imp {
    use super::*;

    #[derive(Debug, Default, CompositeTemplate)]
    #[template(resource = "/org/gnome/Nutmeg/training/ui/page.ui")]
    pub struct TrainingPlannerPage {
        #[template_child]
        pub plans_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub new_plan_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub delete_plan_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub plan_name_entry: TemplateChild<gtk::Entry>,
        #[template_child]
        pub plan_description_label: TemplateChild<gtk::Label>,
        #[template_child]
        pub steps_container: TemplateChild<gtk::Box>,
        #[template_child]
        pub add_step_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub players_list: TemplateChild<gtk::ListBox>,
        #[template_child]
        pub simulate_button: TemplateChild<gtk::Button>,
        #[template_child]
        pub results_label: TemplateChild<gtk::Label>,

        // State
        pub current_plan: RefCell<Option<CyclePlan>>,
        pub plans: RefCell<Vec<CyclePlan>>,
        pub context_object: RefCell<Option<crate::ui::context_object::ContextObject>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for TrainingPlannerPage {
        const NAME: &'static str = "TrainingPlannerPage";
        type Type = super::TrainingPlannerPage;
        type ParentType = gtk::Box;

        fn class_init(klass: &mut Self::Class) {
            klass.bind_template();
        }

        fn instance_init(obj: &glib::subclass::InitializingObject<Self>) {
            obj.init_template();
        }
    }

    impl ObjectImpl for TrainingPlannerPage {
        fn constructed(&self) {
            self.parent_constructed();
            let obj = self.obj();

            // Initial load
            obj.load_plans();

            // Signals
            let obj_weak = obj.downgrade();
            self.new_plan_button.connect_clicked(move |_| {
                if let Some(obj) = obj_weak.upgrade() {
                    obj.create_new_plan();
                }
            });

            let obj_weak = obj.downgrade();
            self.plans_list.connect_row_selected(move |_, row| {
                if let Some(obj) = obj_weak.upgrade() {
                    if let Some(row) = row {
                        let index = row.index() as usize;
                        obj.select_plan(index);
                    }
                }
            });

            // Save on name change
            let obj_weak = obj.downgrade();
            self.plan_name_entry.connect_changed(move |entry| {
                if let Some(obj) = obj_weak.upgrade() {
                    let name = entry.text().to_string();
                    obj.update_current_plan_name(name);
                }
            });

            // Add Step
            let obj_weak = obj.downgrade();
            self.add_step_button.connect_clicked(move |_| {
                if let Some(obj) = obj_weak.upgrade() {
                    obj.add_step();
                }
            });

            // Simulate
            let obj_weak = obj.downgrade();
            self.simulate_button.connect_clicked(move |_| {
                if let Some(obj) = obj_weak.upgrade() {
                    obj.run_simulation();
                }
            });

            // Delete Plan
            let obj_weak = obj.downgrade();
            self.delete_plan_button.connect_clicked(move |_| {
                if let Some(obj) = obj_weak.upgrade() {
                    obj.delete_current_plan();
                }
            });
        }
    }
    impl WidgetImpl for TrainingPlannerPage {}
    impl BoxImpl for TrainingPlannerPage {}
}

glib::wrapper! {
    pub struct TrainingPlannerPage(ObjectSubclass<imp::TrainingPlannerPage>)
        @extends gtk::Widget, gtk::Box,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget, gtk::Orientable;
}

impl Default for TrainingPlannerPage {
    fn default() -> Self {
        Self::new()
    }
}

impl TrainingPlannerPage {
    pub fn new() -> Self {
        glib::Object::builder().build()
    }

    pub fn load_plans(&self) {
        let imp = self.imp();
        let plans = TrainingService::load_plans();
        log::info!("Loaded {} plans", plans.len());
        *imp.plans.borrow_mut() = plans.clone();

        // Populate list
        // Clear first (naive approach)
        while let Some(child) = imp.plans_list.first_child() {
            imp.plans_list.remove(&child);
        }

        for plan in &plans {
            let label = gtk::Label::new(Some(&plan.cycle.name));
            label.set_halign(gtk::Align::Start);
            label.set_margin_start(6);
            label.set_margin_end(6);
            label.set_margin_top(6);
            label.set_margin_bottom(6);
            imp.plans_list.append(&label);
        }

        // Select first if available
        if !plans.is_empty() {
            imp.plans_list
                .select_row(imp.plans_list.row_at_index(0).as_ref());
        }
    }

    pub fn create_new_plan(&self) {
        let new_plan = CyclePlan::default();
        if let Err(e) = TrainingService::save_plan(&new_plan) {
            log::error!("Failed to save new plan: {}", e);
            return;
        }
        self.load_plans();
        // Select the last one (newly created) - simplistic logic for now
        let imp = self.imp();
        let count = imp.plans.borrow().len();
        if count > 0 {
            imp.plans_list
                .select_row(imp.plans_list.row_at_index((count - 1) as i32).as_ref());
        }
    }

    pub fn delete_current_plan(&self) {
        let imp = self.imp();
        let plan_id_to_delete = {
            let plan_opt = imp.current_plan.borrow();
            plan_opt.as_ref().map(|p| p.id)
        };

        if let Some(id) = plan_id_to_delete {
            if let Err(e) = TrainingService::delete_plan(id) {
                log::error!("Failed to delete plan: {}", e);
                return;
            }
            self.load_plans();
        }
    }

    pub fn select_plan(&self, index: usize) {
        let imp = self.imp();
        let plans = imp.plans.borrow();
        if let Some(plan) = plans.get(index) {
            *imp.current_plan.borrow_mut() = Some(plan.clone());
            imp.plan_name_entry.set_text(&plan.cycle.name);
            imp.plan_description_label
                .set_label(&plan.cycle.description);
            self.refresh_steps_ui();
            self.refresh_player_selection_ui();
        }
    }

    pub fn update_current_plan_name(&self, name: String) {
        let imp = self.imp();
        let mut current_opt = imp.current_plan.borrow_mut();
        if let Some(ref mut plan) = *current_opt {
            plan.cycle.name = name;
            if let Err(e) = TrainingService::save_plan(plan) {
                log::error!("Failed to save plan update: {}", e);
            } else {
                // Update list label without full reload?
                // For now, full reload is safer/easier
                // But full reload resets selection, so maybe update UI directly
                if let Some(row) = imp.plans_list.selected_row() {
                    if let Some(child) = row.child() {
                        if let Some(label) = child.downcast_ref::<gtk::Label>() {
                            label.set_label(&plan.cycle.name);
                        }
                    }
                }
            }
        }
    }

    pub fn add_step(&self) {
        let imp = self.imp();
        let mut current_opt = imp.current_plan.borrow_mut();
        if let Some(ref mut plan) = *current_opt {
            plan.cycle.steps.push(CycleStep {
                skill: PlayerSkill::Defending,
                target_level: 8, // Default to Excellent?
                duration_weeks: Some(10),
            });
            if let Err(e) = TrainingService::save_plan(plan) {
                log::error!("Failed to save plan after adding step: {}", e);
            }
        }
        // Release borrow before refresh
        drop(current_opt);
        self.refresh_steps_ui();
    }

    fn refresh_steps_ui(&self) {
        let imp = self.imp();

        // Clear existing steps
        while let Some(child) = imp.steps_container.first_child() {
            imp.steps_container.remove(&child);
        }

        let plan_opt = imp.current_plan.borrow();
        if let Some(plan) = plan_opt.as_ref() {
            for (index, step) in plan.cycle.steps.iter().enumerate() {
                let row = gtk::Box::new(gtk::Orientation::Horizontal, 6);

                // Training Type Combo
                let type_combo = gtk::DropDown::from_strings(&[
                    "Goalkeeping",
                    "Defending",
                    "Playmaking",
                    "Winger",
                    "Passing",
                    "Scoring",
                    "Set Pieces",
                ]);
                // Select current type
                let type_idx = match step.skill {
                    PlayerSkill::Keeper => 0,
                    PlayerSkill::Defending => 1,
                    PlayerSkill::Playmaking => 2,
                    PlayerSkill::Winger => 3,
                    PlayerSkill::Passing => 4,
                    PlayerSkill::Scoring => 5,
                    PlayerSkill::SetPieces => 6,
                    _ => 1, // Default to Defending if unknown
                };
                type_combo.set_selected(type_idx);

                // Signal to update type
                let obj_weak = self.downgrade();
                let idx_copy = index;
                type_combo.connect_selected_notify(move |combo| {
                    if let Some(obj) = obj_weak.upgrade() {
                        let selected = combo.selected();
                        let new_skill = match selected {
                            0 => PlayerSkill::Keeper,
                            1 => PlayerSkill::Defending,
                            2 => PlayerSkill::Playmaking,
                            3 => PlayerSkill::Winger,
                            4 => PlayerSkill::Passing,
                            5 => PlayerSkill::Scoring,
                            6 => PlayerSkill::SetPieces,
                            _ => PlayerSkill::Defending,
                        };
                        obj.update_step_type(idx_copy, new_skill);
                    }
                });
                row.append(&type_combo);

                let until_label = gtk::Label::new(Some("until level"));
                row.append(&until_label);

                // Target Level SpinButton
                let level_adj =
                    gtk::Adjustment::new(step.target_level as f64, 1.0, 20.0, 1.0, 5.0, 0.0);
                let level_spin = gtk::SpinButton::new(Some(&level_adj), 1.0, 0);
                let obj_weak = self.downgrade();
                let idx_copy = index;
                level_spin.connect_value_changed(move |spin| {
                    if let Some(obj) = obj_weak.upgrade() {
                        obj.update_step_target_level(idx_copy, spin.value() as u8);
                    }
                });
                row.append(&level_spin);

                let spacer = gtk::Label::new(Some("(approx."));
                spacer.set_margin_start(12);
                row.append(&spacer);

                // Weeks SpinButton
                let weeks_val = step.duration_weeks.unwrap_or(1) as f64;
                let weeks_adj = gtk::Adjustment::new(weeks_val, 1.0, 104.0, 1.0, 5.0, 0.0);
                let weeks_spin = gtk::SpinButton::new(Some(&weeks_adj), 1.0, 0);
                // Signal to update weeks
                let obj_weak = self.downgrade();
                let idx_copy = index;
                weeks_spin.connect_value_changed(move |spin| {
                    if let Some(obj) = obj_weak.upgrade() {
                        obj.update_step_weeks(idx_copy, spin.value() as u32);
                    }
                });
                row.append(&weeks_spin);

                let weeks_label = gtk::Label::new(Some("weeks)"));
                row.append(&weeks_label);

                // Remove Button
                let remove_btn = gtk::Button::from_icon_name("user-trash-symbolic");
                let obj_weak = self.downgrade();
                let idx_copy = index;
                remove_btn.connect_clicked(move |_| {
                    if let Some(obj) = obj_weak.upgrade() {
                        obj.remove_step(idx_copy);
                    }
                });
                row.append(&remove_btn);

                imp.steps_container.append(&row);
            }
        }
    }

    pub fn set_context_object(&self, context: &crate::ui::context_object::ContextObject) {
        let imp = self.imp();
        *imp.context_object.borrow_mut() = Some(context.clone());

        // Listen for player updates
        let obj_weak = self.downgrade();
        context.connect_notify_local(Some("players"), move |_, _| {
            if let Some(obj) = obj_weak.upgrade() {
                obj.refresh_player_selection_ui();
            }
        });

        // Initial refresh
        self.refresh_player_selection_ui();
    }

    fn refresh_player_selection_ui(&self) {
        let imp = self.imp();

        // Clear existing
        while let Some(child) = imp.players_list.first_child() {
            imp.players_list.remove(&child);
        }

        let context_opt = imp.context_object.borrow();
        if let Some(context) = context_opt.as_ref() {
            let players_value: glib::Value = context.property("players");
            if let Ok(Some(store)) = players_value.get::<Option<gtk::ListStore>>() {
                let mut iter = store.iter_first();
                let mut count = 0;
                while let Some(it) = iter {
                    let player_obj: crate::ui::player_object::PlayerObject = store.get(&it, 18);
                    if true {
                        // preserving scope for diff simplicity

                        let player = player_obj.player();
                        let player_id = player.PlayerID;
                        let name = format!("{} {}", player.FirstName, player.LastName);

                        let row_box = gtk::Box::new(gtk::Orientation::Horizontal, 6);
                        let check = gtk::CheckButton::new();
                        count += 1;

                        // Check if selected in current plan
                        let is_selected = if let Some(plan) = imp.current_plan.borrow().as_ref() {
                            plan.trainee_ids.contains(&player_id)
                        } else {
                            false
                        };
                        check.set_active(is_selected);

                        let label = gtk::Label::new(Some(&name));
                        row_box.append(&check);
                        row_box.append(&label);

                        imp.players_list.append(&row_box);

                        // Connect signal
                        let obj_weak = self.downgrade();
                        check.connect_toggled(move |btn| {
                            if let Some(obj) = obj_weak.upgrade() {
                                obj.toggle_player_selection(player_id, btn.is_active());
                            }
                        });
                    }
                    if !store.iter_next(&it) {
                        break;
                    }
                    iter = Some(it);
                }
                log::info!("Refreshed player list with {} players", count);
            }
        } else {
            log::warn!("Context object missing in refresh_player_selection_ui");
        }
    }

    pub fn toggle_player_selection(&self, player_id: u32, is_active: bool) {
        let imp = self.imp();
        let mut current_opt = imp.current_plan.borrow_mut();
        if let Some(ref mut plan) = *current_opt {
            if is_active {
                if !plan.trainee_ids.contains(&player_id) {
                    plan.trainee_ids.push(player_id);
                }
            } else {
                plan.trainee_ids.retain(|&id| id != player_id);
            }
            if let Err(e) = TrainingService::save_plan(plan) {
                log::error!("Failed to save plan update: {}", e);
            }
        }
    }

    pub fn run_simulation(&self) {
        let imp = self.imp();
        let plan_opt = imp.current_plan.borrow();
        let plan = match plan_opt.as_ref() {
            Some(p) => p,
            None => {
                imp.results_label.set_label("No plan selected");
                return;
            }
        };

        if plan.trainee_ids.is_empty() {
            imp.results_label
                .set_label("No trainees selected. Select players in the list below to simulate.");
            return;
        }

        // Fetch players from context
        let mut trainees = Vec::new();
        let context_opt = imp.context_object.borrow();
        if let Some(context) = context_opt.as_ref() {
            let players_value: glib::Value = context.property("players");
            if let Ok(Some(store)) = players_value.get::<Option<gtk::ListStore>>() {
                let mut iter = store.iter_first();
                while let Some(it) = iter {
                    let player_obj: crate::ui::player_object::PlayerObject = store.get(&it, 18);
                    let player = player_obj.player();
                    if plan.trainee_ids.contains(&player.PlayerID) {
                        trainees.push(player.clone());
                    }
                    if !store.iter_next(&it) {
                        break;
                    }
                    iter = Some(it);
                }
            }
        }

        if trainees.is_empty() {
            imp.results_label
                .set_label("Selected trainees not found in current squad.");
            return;
        }

        let progress = TrainingService::calculate_progress(plan, &trainees);

        // Summarise results
        let mut summary = String::from("Simulation Results:\n\n");
        for trainee in &trainees {
            let trainee_progress: Vec<_> = progress
                .iter()
                .filter(|p| p.player_id == trainee.PlayerID)
                .collect();
            if let Some(last) = trainee_progress.last() {
                let years = last.week / 16;
                let weeks = last.week % 16;
                summary.push_str(&format!(
                    "• {}: Reaches target in {} weeks ({} seasons, {} weeks). Final level: {:.1}\n",
                    trainee.FirstName, last.week, years, weeks, last.level
                ));
            }
        }

        imp.results_label.set_label(&summary);
    }

    #[allow(dead_code)]
    fn display_results(&self) {
        // Placeholder
    }

    pub fn update_step_type(&self, index: usize, new_skill: PlayerSkill) {
        let imp = self.imp();
        let mut current_opt = imp.current_plan.borrow_mut();
        if let Some(ref mut plan) = *current_opt {
            if let Some(step) = plan.cycle.steps.get_mut(index) {
                step.skill = new_skill;
                if let Err(e) = TrainingService::save_plan(plan) {
                    log::error!("Failed to save plan after type update: {}", e);
                }
            }
        }
    }

    pub fn update_step_weeks(&self, index: usize, weeks: u32) {
        let imp = self.imp();
        let mut current_opt = imp.current_plan.borrow_mut();
        if let Some(ref mut plan) = *current_opt {
            if let Some(step) = plan.cycle.steps.get_mut(index) {
                step.duration_weeks = Some(weeks);
                if let Err(e) = TrainingService::save_plan(plan) {
                    log::error!("Failed to save plan after duration update: {}", e);
                }
            }
        }
    }

    pub fn update_step_target_level(&self, index: usize, level: u8) {
        let imp = self.imp();
        let mut current_opt = imp.current_plan.borrow_mut();
        if let Some(ref mut plan) = *current_opt {
            if let Some(step) = plan.cycle.steps.get_mut(index) {
                step.target_level = level;
                if let Err(e) = TrainingService::save_plan(plan) {
                    log::error!("Failed to save plan after target level update: {}", e);
                }
            }
        }
    }

    pub fn remove_step(&self, index: usize) {
        let imp = self.imp();
        let mut current_opt = imp.current_plan.borrow_mut();
        if let Some(ref mut plan) = *current_opt {
            if index < plan.cycle.steps.len() {
                plan.cycle.steps.remove(index);
                if let Err(e) = TrainingService::save_plan(plan) {
                    log::error!("Failed to save plan after step removal: {}", e);
                }
            }
        }
        // Release borrow before refresh
        drop(current_opt);
        self.refresh_steps_ui();
    }
}
