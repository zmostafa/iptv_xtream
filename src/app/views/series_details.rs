use crate::api::fetch_serie_info;
use crate::app::state::{AppView, IPTVApp};
use crate::utils;
use eframe::egui;

pub fn render_series_details(
    app: &mut IPTVApp,
    ctx: &egui::Context,
    category_id: &str,
    series_id: &i64,
    category_name: &str,
) {
    let series_detail = app
        .db
        .get_series_info(category_id, series_id)
        .unwrap_or_else(|| {
            let serie = futures::executor::block_on(fetch_serie_info(
                &app.client,
                &app.api_url,
                &app.username,
                &app.password,
                series_id,
            ))
            .unwrap();
            app.db.save_series_info(&category_id, &series_id, &serie);
            serie
        });

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading(format!(
            "Series: {}",
            utils::preprocess_arabic_text_v1(series_detail.info.name.as_str())
        ));
        ui.label(format!(
            "Plot: {}",
            utils::preprocess_arabic_text_v1(series_detail.info.plot.as_str())
        ));

        if ui.button("Back").clicked() {
            app.current_view = AppView::SeriesList(
                series_detail.info.category_id.clone(),
                category_name.to_string(),
            );
        }

        // Convert episodes keys to a sorted vector
        // HashMap does not maintain order,so looping through them will change how they are displayed
        let mut sorted_seasons: Vec<_> = series_detail.episodes.keys().cloned().collect();
        sorted_seasons.sort();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for season in sorted_seasons {
                if ui.button(format!("Season {}", season)).clicked() {
                    app.current_view = AppView::EpisodeList(
                        category_id.to_string(),
                        series_id.to_owned(),
                        season,
                        category_name.to_string(),
                    );
                }
            }
        });
    });
}
