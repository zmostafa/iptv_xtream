use crate::app::state::{AppView, IPTVApp};
use eframe::egui;
use egui::{Pos2, Rect};
pub fn render_categories(app: &mut IPTVApp, ctx: &egui::Context) {
    egui::CentralPanel::default().show(ctx, |ui| {
        // Draw the background image
        let rect = Rect::from_min_size(Pos2::ZERO, ui.available_size());
        ui.painter().image(
            app.background.texture_id(ctx),
            rect,
            egui::Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), // UV coordinates
            egui::Color32::WHITE, // Tint color (use WHITE for no tint)
        );
        ui.vertical(|ui| {
            // ui.centered_and_justified(|ui| {

            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                ui.horizontal(|ui| {
                    app.view_stack.push(app.current_view.clone());

                    let current_dir = std::env::current_dir().unwrap();

                    let movies_icon = egui_extras::image::load_svg_bytes_with_size(
                        &std::fs::read(current_dir.join("assets/movies.svg"))
                            .expect("Failed to read movie icon"),
                        Some(egui::SizeHint::Size(150, 150)),
                    );

                    let series_icon = egui_extras::image::load_svg_bytes_with_size(
                        &std::fs::read(current_dir.join("assets/series.svg"))
                            .expect("Failed to read movie icon"),
                        Some(egui::SizeHint::Size(150, 150)),
                    );

                    let tv_icon = egui_extras::image::load_svg_bytes_with_size(
                        &std::fs::read(current_dir.join("assets/tv.svg"))
                            .expect("Failed to read movie icon"),
                        Some(egui::SizeHint::Size(150, 150)),
                    );

                    match tv_icon {
                        Ok(icon) => {
                            let texture = ctx.load_texture("icon", icon, Default::default());
                            if ui.add(egui::ImageButton::new(&texture)).clicked() {
                                app.current_view = AppView::LiveCategories;
                            }
                        }
                        Err(e) => {
                            ui.label(format!("Error loading image: {:?}", e));
                        }
                    }

                    match movies_icon {
                        Ok(icon) => {
                            let texture = ctx.load_texture("icon", icon, Default::default());
                            if ui.add(egui::ImageButton::new(&texture)).clicked() {
                                app.current_view = AppView::MoviesCategories;
                            }
                        }
                        Err(e) => {
                            ui.label(format!("Error loading image: {:?}", e));
                        }
                    }

                    match series_icon {
                        Ok(icon) => {
                            let texture = ctx.load_texture("icon", icon, Default::default());
                            if ui.add(egui::ImageButton::new(&texture)).clicked() {
                                app.current_view = AppView::SeriesCategories;
                            }
                        }
                        Err(e) => {
                            ui.label(format!("Error loading image: {:?}", e));
                        }
                    }
                });
            });
            // });
            ui.separator();
            ui.centered_and_justified(|ui| {
                ui.horizontal(|ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        let current_dir = std::env::current_dir().unwrap();
                        let update_icon = egui_extras::image::load_svg_bytes_with_size(
                            &std::fs::read(current_dir.join("assets/update.svg"))
                                .expect("Failed to read update icon"),
                            Some(egui::SizeHint::Size(150, 150)),
                        );

                        match update_icon {
                            Ok(icon) => {
                                let texture = ctx.load_texture("icon", icon, Default::default());
                                if ui.add(egui::ImageButton::new(&texture)).clicked() {
                                    futures::executor::block_on(
                                        app.fetch_and_save_all_live_streams(),
                                    );
                                }
                            }
                            Err(e) => {
                                ui.label(format!("Error loading image: {:?}", e));
                            }
                        }
                    });

                    // if ui.button("Fetch Content").clicked() {
                    //     futures::executor::block_on(app.fetch_and_save_all_live_streams());
                    // }
                });
            });
        });
    });
}
