use crate::{
    app::state::{AppView, IPTVApp},
    utils,
};
use eframe::egui;

pub fn render_series_categories(app: &mut IPTVApp, ctx: &egui::Context) {
    let categories = app.db.get_series_categories();
    app.series_cache.clear();

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("Series Categories");

        if ui.button("Search Series").clicked() {
            app.current_view = AppView::SeriesSearch;
        }

        if ui.button("Recently watched series").clicked() {
            app.view_stack.push(app.current_view.clone());
            app.current_view = AppView::RecentlyWatchedSeries;
        }

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
                        AppView::SeriesList(category.category_id.clone(), display_name.clone());
                }
            }
        });
    });
}
