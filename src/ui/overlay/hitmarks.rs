use std::{
    cell::Cell,
    fs::File,
    io::BufReader,
    sync::Arc,
    time::{Duration, Instant},
};

use egui::{Color32, Painter, Stroke, pos2};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source, buffer::SamplesBuffer};

use crate::{data::Data, ui::app::App};

#[derive(Debug, Default)]
pub struct HitmarkState {
    last_hit: Cell<Option<Instant>>,
    previous_total_hits: Cell<Option<i32>>,
}

impl HitmarkState {
    fn update(&self, total_hits: i32, now: Instant) -> bool {
        let hit = self
            .previous_total_hits
            .get()
            .is_some_and(|previous_total_hits| {
                total_hits != previous_total_hits && total_hits != 0
            });

        self.previous_total_hits.set(Some(total_hits));

        if hit {
            self.last_hit.set(Some(now));
        }

        hit
    }

    fn alpha(&self, now: Instant, fadeout_duration: Duration) -> Option<f32> {
        let last_hit = self.last_hit.get()?;
        let elapsed = now.saturating_duration_since(last_hit);

        if elapsed >= fadeout_duration {
            return None;
        }

        let fadeout_secs = fadeout_duration.as_secs_f32();
        if fadeout_secs <= f32::EPSILON {
            return None;
        }

        Some((1.0 - elapsed.as_secs_f32() / fadeout_secs).clamp(0.0, 1.0))
    }
}

pub struct HitsoundPlayer {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    tracks: [Option<Hitsound>; 3],
}

impl HitsoundPlayer {
    pub fn new() -> Option<Self> {
        let (_stream, stream_handle) = OutputStream::try_default().ok()?;

        Some(Self {
            _stream,
            stream_handle,
            tracks: [
                Hitsound::from_file("resources/sound/hitsound1.wav"),
                Hitsound::from_file("resources/sound/hitsound2.wav"),
                Hitsound::from_file("resources/sound/hitsound3.wav"),
            ],
        })
    }

    fn play(&self, track: i32) {
        let Some(index) = usize::try_from(track)
            .ok()
            .and_then(|track| track.checked_sub(1))
        else {
            return;
        };

        let Some(Some(sound)) = self.tracks.get(index) else {
            return;
        };

        let Ok(sink) = Sink::try_new(&self.stream_handle) else {
            return;
        };

        sink.append(sound.source());
        sink.detach();
    }
}

#[derive(Debug, Clone)]
struct Hitsound {
    channels: u16,
    sample_rate: u32,
    samples: Arc<[f32]>,
}

impl Hitsound {
    fn from_file(path: &str) -> Option<Self> {
        let file = File::open(path).ok()?;
        let decoder = Decoder::new(BufReader::new(file)).ok()?;

        let channels = decoder.channels();
        let sample_rate = decoder.sample_rate();
        let samples = decoder.convert_samples::<f32>().collect::<Vec<_>>().into();

        Some(Self {
            channels,
            sample_rate,
            samples,
        })
    }

    fn source(&self) -> SamplesBuffer<f32> {
        SamplesBuffer::new(self.channels, self.sample_rate, self.samples.to_vec())
    }
}

impl App {
    pub fn draw_hitmarks(&self, painter: &Painter, data: &Data) {
        if data.local_player.health <= 0 {
            return;
        }

        let config = &self.config.hitmarks;
        let now = Instant::now();
        let total_hits = data.local_player.total_hits;

        let hit = self.hitmark_state.update(total_hits, now);

        if hit && config.hitsound_enabled {
            if let Some(player) = &self.hitsound_player {
                player.play(config.hitsound_track);
            }
        }

        if !config.hitmark_enabled {
            return;
        }

        let fadeout_duration = Duration::from_secs_f32(config.fadeout_duration.max(0.0));

        let Some(alpha) = self.hitmark_state.alpha(now, fadeout_duration) else {
            return;
        };

        let alpha = (alpha * 255.0) as u8;
        let color = Color32::from_rgba_unmultiplied(
            config.color.r(),
            config.color.g(),
            config.color.b(),
            alpha,
        );
        let stroke = Stroke::new(self.config.hud.line_width.max(1.0), color);

        let center_x = data.window_size.x / 2.0;
        let center_y = data.window_size.y / 2.0;

        let gap = config.gap.max(0.0);
        let size = config.size.max(gap);

        painter.line_segment(
            [
                pos2(center_x + gap, center_y + gap),
                pos2(center_x + size, center_y + size),
            ],
            stroke,
        );

        painter.line_segment(
            [
                pos2(center_x + gap, center_y - gap),
                pos2(center_x + size, center_y - size),
            ],
            stroke,
        );

        painter.line_segment(
            [
                pos2(center_x - gap, center_y + gap),
                pos2(center_x - size, center_y + size),
            ],
            stroke,
        );

        painter.line_segment(
            [
                pos2(center_x - gap, center_y - gap),
                pos2(center_x - size, center_y - size),
            ],
            stroke,
        );
    }
}
