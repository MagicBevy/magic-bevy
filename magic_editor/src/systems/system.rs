use eframe::egui;

pub trait EditorSystem: Send + Sync {
    fn on_enable(&mut self);

    fn on_update(&mut self, ctx: &egui::Context, ui: &mut egui::Ui);

    fn on_disable(&mut self);
}