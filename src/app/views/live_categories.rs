use crate::{
    app::state::{AppView, IPTVApp},
    utils,
};
use eframe::egui;

pub fn render_live_categories(app: &mut IPTVApp, ctx: &egui::Context) {
    let categories = app.db.get_live_categories();

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Live Categories");

        if ui.button("Back").clicked() {
            let view = app.view_stack.pop().unwrap();
            app.current_view = view;
        }

        egui::ScrollArea::vertical().show(ui, |ui| {
            for category in categories {
                let display_name = utils::preprocess_arabic_text_v1(&category.category_name);
                if ui.button(&display_name).clicked() {
                    app.view_stack.push(app.current_view.clone());
                    app.current_view =
                        AppView::LiveStreams(category.category_id.clone(), display_name.clone());
                }
            }
        });
    });
}
