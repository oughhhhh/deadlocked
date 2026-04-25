use egui::{DragValue, Ui};

use crate::ui::{
    app::App,
    gui::helpers::{checkbox, checkbox_hover, collapsing_open, drag, scroll},
};

impl App {
    pub fn radar_settings(&mut self, ui: &mut Ui) {
        ui.columns(2, |cols| {
            let left = &mut cols[0];
            scroll(left, "radar_left", |ui| self.radar_left(ui));

            let right = &mut cols[1];
            scroll(right, "radar_right", |ui| self.radar_right(ui));
        });
    }

    fn radar_left(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Radar", |ui| {
            if checkbox(ui, "Enable Radar", &mut self.config.radar.enabled) {
                self.send_config();
            }

            if checkbox_hover(
                ui,
                "Show Friendlies",
                "Only useful in custom game modes or when friendly player data is available",
                &mut self.config.radar.show_friendlies,
            ) {
                self.send_config();
            }

            if checkbox_hover(
                ui,
                "Direction Cones",
                "Draws a small white direction marker in front of each radar dot",
                &mut self.config.radar.cones,
            ) {
                self.send_config();
            }
        });

        ui.collapsing("Size", |ui| {
            if drag(
                ui,
                "Radar Size",
                DragValue::new(&mut self.config.radar.size)
                    .range(0.25..=3.0)
                    .speed(0.02)
                    .max_decimals(2),
            ) {
                self.send_config();
            }

            if drag(
                ui,
                "Radar Zoom",
                DragValue::new(&mut self.config.radar.zoom)
                    .range(0.25..=5.0)
                    .speed(0.02)
                    .max_decimals(2),
            ) {
                self.send_config();
            }

            if drag(
                ui,
                "Dot Radius",
                DragValue::new(&mut self.config.radar.dot_radius)
                    .range(1.0..=24.0)
                    .speed(0.1)
                    .max_decimals(1),
            ) {
                self.send_config();
            }

            if drag(
                ui,
                "Distance Limit",
                DragValue::new(&mut self.config.radar.distance_limit)
                    .range(0.01..=1.0)
                    .speed(0.002)
                    .max_decimals(3),
            ) {
                self.send_config();
            }
        });
    }

    fn radar_right(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Position", |ui| {
            if drag(
                ui,
                "Margin X",
                DragValue::new(&mut self.config.radar.margin_x)
                    .range(0.0..=4000.0)
                    .speed(1.0)
                    .max_decimals(0),
            ) {
                self.send_config();
            }

            if drag(
                ui,
                "Margin Y",
                DragValue::new(&mut self.config.radar.margin_y)
                    .range(0.0..=4000.0)
                    .speed(1.0)
                    .max_decimals(0),
            ) {
                self.send_config();
            }
        });

        ui.collapsing("Style", |ui| {
            if drag(
                ui,
                "Background Alpha",
                DragValue::new(&mut self.config.radar.background_alpha)
                    .range(0..=255)
                    .speed(1.0),
            ) {
                self.send_config();
            }

            if drag(
                ui,
                "Border Alpha",
                DragValue::new(&mut self.config.radar.border_alpha)
                    .range(0..=255)
                    .speed(1.0),
            ) {
                self.send_config();
            }
        });
    }
}
