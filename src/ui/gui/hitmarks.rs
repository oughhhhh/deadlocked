use crate::ui::{
    app::App,
    gui::helpers::{checkbox_hover, collapsing_open, drag},
};
use egui::{
    DragValue, Ui,
    color_picker::{Alpha, color_edit_button_srgba},
};

impl App {
    pub fn hitmarks_settings(&mut self, ui: &mut Ui) {
        collapsing_open(ui, "Hitmarks", |ui| {
            if checkbox_hover(
                ui,
                "Hitmarks Enabled",
                "Draw a hitmarker when a hit is registered",
                &mut self.config.hitmarks.hitmark_enabled,
            ) {
                self.send_config();
            }

            if checkbox_hover(
                ui,
                "Hitsound Enabled",
                "Play a sound when a hit is registered",
                &mut self.config.hitmarks.hitsound_enabled,
            ) {
                self.send_config();
            }

            ui.horizontal(|ui| {
                ui.label("Color");
                if color_edit_button_srgba(ui, &mut self.config.hitmarks.color, Alpha::OnlyBlend)
                    .changed()
                {
                    self.send_config();
                }
            });

            if drag(
                ui,
                "Fadeout Time (s)",
                DragValue::new(&mut self.config.hitmarks.fadeout_duration)
                    .range(0.0..=2.0)
                    .speed(0.01),
            ) {
                self.send_config();
            }

            if drag(
                ui,
                "Gap",
                DragValue::new(&mut self.config.hitmarks.gap)
                    .range(0.0..=30.0)
                    .speed(0.1),
            ) {
                self.send_config();
            }

            if drag(
                ui,
                "Size",
                DragValue::new(&mut self.config.hitmarks.size)
                    .range(1.0..=50.0)
                    .speed(0.1),
            ) {
                self.send_config();
            }

            if self.config.hitmarks.hitsound_enabled {
                if drag(
                    ui,
                    "Hitsound Track",
                    DragValue::new(&mut self.config.hitmarks.hitsound_track).range(1..=3),
                ) {
                    self.send_config();
                }
                ui.label("Hitsound Track: 1-3");
            }
        });
    }
}
