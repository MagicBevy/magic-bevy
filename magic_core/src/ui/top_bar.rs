use eframe::egui;
use crate::api::MenuRegistry;
use crate::status::CompileStatus;
use crate::ui::indicator::draw_status_indicator;

pub fn draw_top_bar(ctx: &egui::Context, registry: &MenuRegistry, compile_status: &CompileStatus) {
    egui::TopBottomPanel::top("main_top_bar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            
            for (menu_name, commands) in &registry.menus {
                ui.menu_button(menu_name, |ui| {
                    for cmd in commands {
                        if ui.button(&cmd.label).clicked() {
                            (cmd.callback)();
                            ui.close_menu();
                        }
                    }
                });
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                draw_status_indicator(ctx, ui, compile_status);
            });
        });
    });
}