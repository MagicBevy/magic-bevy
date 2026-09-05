// editor/src/ui/indicator.rs
use eframe::egui;
use crate::status::CompileStatus;

pub fn draw_status_indicator(ctx: &egui::Context, ui: &mut egui::Ui, status: &CompileStatus) {
    ui.horizontal(|ui| {
        ui.add_space(5.0);
        
        match status {
            CompileStatus::Idle => {
                draw_led(ui, egui::Color32::from_gray(100));
                ui.weak("Ready");
            }
            CompileStatus::Compiling { current_file, progress } => {
                let time = ui.input(|i| i.time);
                let alpha = (((time * 5.0).sin() + 1.0) / 2.0 * 255.0) as u8;
                
                let blue_blink = egui::Color32::from_rgba_unmultiplied(0, 150, 255, alpha);
                
                draw_led(ui, blue_blink);
                ctx.request_repaint();
                
                ui.small(format!("[{}] Indexing: {}", progress, current_file));
            }
            CompileStatus::Success => {
                draw_led(ui, egui::Color32::from_rgb(50, 200, 50));
                ui.colored_label(egui::Color32::from_rgb(50, 200, 50), "Active");
            }
            CompileStatus::Error(err_msg) => {
                draw_led(ui, egui::Color32::from_rgb(220, 50, 50));
                ui.colored_label(egui::Color32::from_rgb(220, 50, 50), "Error").on_hover_text(err_msg);
            }
        }
    });
}

fn draw_led(ui: &mut egui::Ui, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), 5.0, color);
}