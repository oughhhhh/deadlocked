use std::{
    cell::RefCell,
    process::Command,
    thread,
};

use egui::{pos2, Color32, Painter, Stroke};

use crate::{data::Data, ui::app::App};

const HITMARKS_ENABLED: bool = true;
const HITMARK_FRAMES: i32 = 200;
const HITSOUND_TRACK: i32 = 1;

thread_local! {
    static HITMARK_STATE: RefCell<HitmarkState> =
        RefCell::new(HitmarkState::default());
}

#[derive(Debug, Default)]
struct HitmarkState {
    current_frame: i32,
    last_frame: i32,
    previous_total_hits: Option<i32>,
}

impl HitmarkState {
    fn to_add_hitmark(&mut self, frames: i32, total_hits: i32, hitsound_track: i32) -> bool {
        let Some(previous_total_hits) = self.previous_total_hits else {
            self.previous_total_hits = Some(total_hits);
            return false;
        };

        if total_hits != previous_total_hits && total_hits != 0 {
            self.previous_total_hits = Some(total_hits);
            self.last_frame = self.current_frame + frames;

            if hitsound_track != 0 {
                thread::spawn(move || play_hitsound(hitsound_track));
            }

            return true;
        }

        if self.current_frame < self.last_frame {
            self.previous_total_hits = Some(total_hits);
            return true;
        }

        self.current_frame = 0;
        self.last_frame = 0;

        false
    }
}

impl App {
    pub fn draw_hitmarks(&self, painter: &Painter, data: &Data) {
        let draw_frames = self.hitmark_frames().max(1);
        let total_hits = data.local_player.total_hits;

        if data.local_player.health <= 0 {
            return;
        }

        let alpha = HITMARK_STATE.with(|state| {
            let mut state = state.borrow_mut();

            state.current_frame += 1;

            if !state.to_add_hitmark(draw_frames, total_hits, self.hitsound_track()) {
                return None;
            }

            Some(
                ((state.last_frame - state.current_frame).max(0) as f32 / draw_frames as f32)
                    .clamp(0.0, 1.0),
            )
        });

        let Some(alpha) = alpha else {
            return;
        };

        if !self.hitmarks_enabled() {
            return;
        }

        let alpha = (alpha * 255.0) as u8;
        let color = Color32::from_rgba_unmultiplied(255, 255, 255, alpha);
        let stroke = Stroke::new(self.config.hud.line_width.max(1.0), color);

        let center_x = data.window_size.x / 2.0;
        let center_y = data.window_size.y / 2.0;

        const GAP: f32 = 2.0;
        const SIZE: f32 = 10.0;

        painter.line_segment(
            [
                pos2(center_x + GAP, center_y + GAP),
                pos2(center_x + SIZE, center_y + SIZE),
            ],
            stroke,
        );

        painter.line_segment(
            [
                pos2(center_x + GAP, center_y - GAP),
                pos2(center_x + SIZE, center_y - SIZE),
            ],
            stroke,
        );

        painter.line_segment(
            [
                pos2(center_x - GAP, center_y + GAP),
                pos2(center_x - SIZE, center_y + SIZE),
            ],
            stroke,
        );

        painter.line_segment(
            [
                pos2(center_x - GAP, center_y - GAP),
                pos2(center_x - SIZE, center_y - SIZE),
            ],
            stroke,
        );
    }

    fn hitmarks_enabled(&self) -> bool {
        HITMARKS_ENABLED
    }

    fn hitmark_frames(&self) -> i32 {
        HITMARK_FRAMES
    }

    fn hitsound_track(&self) -> i32 {
        HITSOUND_TRACK
    }
}

fn play_hitsound(track: i32) {
    let file = match track {
        1 => "resources/sound/hitsound1.wav",
        2 => "resources/sound/hitsound2.wav",
        3 => "resources/sound/hitsound3.wav",
        _ => return,
    };

    play_sound_file(file);
}

fn play_sound_file(file: &str) {
    for player in ["pw-play", "paplay", "aplay"] {
        if Command::new(player).arg(file).spawn().is_ok() {
            return;
        }
    }
}
