use egui::{Color32, Painter, Pos2, Rect, Stroke, Vec2, pos2, vec2};
use glam::Vec3;

use crate::{
    data::{Data, PlayerData},
    ui::app::App,
};

const MAGIC_RADAR_SCALE: f32 = 1850.0;
const MAGIC_RADAR_CENTER: f32 = 0.1165;
const MAGIC_RADAR_RADIUS_SCALE: f32 = 1.5186;

#[derive(Debug, Clone, Copy)]
struct OverheadMap {
    pos_x: f32,
    pos_y: f32,
    scale: f32,
}

impl OverheadMap {
    const fn new(pos_x: f32, pos_y: f32, scale: f32) -> Self {
        Self {
            pos_x,
            pos_y,
            scale,
        }
    }
}

impl App {
    pub fn draw_radar(&self, painter: &Painter, data: &Data) {
        if !self.config.radar.enabled {
            return;
        }

        if data.local_player.health <= 0 {
            return;
        }

        let Some(map) = overhead_map(&data.map_name) else {
            return;
        };

        let window_height = data.window_size.y.max(1.0);
        let radar_size = self.config.radar.size.max(0.01);
        let radar_zoom = self.config.radar.zoom.max(0.01);
        let dot_radius = self.config.radar.dot_radius.max(1.0);

        let radar_center = radar_size * window_height * MAGIC_RADAR_CENTER;
        let radar_origin = pos2(self.config.radar.margin_x * window_height / 1080.0, self.config.radar.margin_y * window_height / 1080.0);
        let radar_diameter = radar_center * radar_size * MAGIC_RADAR_RADIUS_SCALE;

        let radar_rect = Rect::from_min_size(radar_origin, vec2(radar_diameter, radar_diameter));

        painter.rect(
            radar_rect,
            0,
            Color32::from_rgba_unmultiplied(0, 0, 0, self.config.radar.background_alpha),
            Stroke::new(
                self.config.hud.line_width,
                Color32::from_rgba_unmultiplied(255, 255, 255, self.config.radar.border_alpha),
            ),
            egui::StrokeKind::Middle,
        );

        let local_map_pos = world_to_radar(
            data.local_player.position,
            map,
            window_height,
            radar_size,
            radar_zoom,
        );

        let local_yaw = data.local_player.rotation;

        let local_radar_pos = center_and_rotate(
            local_map_pos,
            local_map_pos,
            local_yaw,
            radar_origin,
            radar_center,
            0.0,
        );

        painter.circle_filled(local_radar_pos, dot_radius, Color32::GREEN);

        let distance_limit = window_height * self.config.radar.distance_limit.max(0.0);

        for player in &data.players {
            if player.health <= 0 {
                continue;
            }

            self.draw_radar_player(
                painter,
                player,
                map,
                local_map_pos,
                local_yaw,
                radar_origin,
                radar_center,
                dot_radius,
                distance_limit,
                window_height,
                radar_size,
                radar_zoom,
                Color32::RED,
            );
        }

        if self.config.radar.show_friendlies && data.is_custom_mode {
            for player in &data.friendlies {
                if player.health <= 0 {
                    continue;
                }

                self.draw_radar_player(
                    painter,
                    player,
                    map,
                    local_map_pos,
                    local_yaw,
                    radar_origin,
                    radar_center,
                    dot_radius,
                    distance_limit,
                    window_height,
                    radar_size,
                    radar_zoom,
                    Color32::YELLOW,
                );
            }
        }

        // @TODO: bomb support
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_radar_player(
        &self,
        painter: &Painter,
        player: &PlayerData,
        map: OverheadMap,
        local_map_pos: Vec2,
        local_yaw: f32,
        radar_origin: Pos2,
        radar_center: f32,
        dot_radius: f32,
        distance_limit: f32,
        window_height: f32,
        radar_size: f32,
        radar_zoom: f32,
        color: Color32,
    ) {
        let map_pos = world_to_radar(
            player.position,
            map,
            window_height,
            radar_size,
            radar_zoom,
        );

        if distance_2d(local_map_pos, map_pos) > distance_limit {
            return;
        }

        let radar_pos = center_and_rotate(
            map_pos,
            local_map_pos,
            local_yaw,
            radar_origin,
            radar_center,
            0.0,
        );

        if self.config.radar.cones {
            let ray = yaw_to_vec(player.rotation);
            let map_ray = map_pos + vec2(dot_radius * ray.x, -dot_radius * ray.y);

            let ray_pos = center_and_rotate(
                map_ray,
                local_map_pos,
                local_yaw,
                radar_origin,
                radar_center,
                0.0,
            );

            painter.circle_filled(ray_pos, dot_radius / 2.0, Color32::WHITE);
            painter.circle_filled(radar_pos, dot_radius, color);
        } else {
            draw_filled_rhomb(painter, radar_pos, dot_radius * 1.2, color);
        }
    }
}

fn draw_filled_rhomb(painter: &Painter, center: Pos2, radius: f32, color: Color32) {
    let points = vec![
        center + vec2(0.0, -radius),
        center + vec2(radius, 0.0),
        center + vec2(0.0, radius),
        center + vec2(-radius, 0.0),
    ];

    painter.add(egui::Shape::convex_polygon(
        points,
        color,
        Stroke::NONE,
    ));
}

fn world_to_radar(
    origin: Vec3,
    map: OverheadMap,
    window_height: f32,
    radar_size: f32,
    radar_zoom: f32,
) -> Vec2 {
    let mut offset = vec2(origin.x - map.pos_x, origin.y - map.pos_y);

    let map_scale = MAGIC_RADAR_SCALE * map.scale / window_height / radar_size / radar_zoom;

    offset.x /= map_scale;
    offset.y /= -map_scale;

    offset
}

fn center_and_rotate(
    map_pos: Vec2,
    local_map_pos: Vec2,
    local_yaw: f32,
    radar_origin: Pos2,
    radar_center: f32,
    angle_offset: f32,
) -> Pos2 {
    let offset = map_pos - local_map_pos;

    let theta = (local_yaw - angle_offset - 90.0).to_radians();
    let cs = theta.cos();
    let sn = theta.sin();

    let px = offset.x * cs - offset.y * sn;
    let py = offset.x * sn + offset.y * cs;

    radar_origin + vec2(px + radar_center, py + radar_center)
}

fn yaw_to_vec(yaw_degrees: f32) -> Vec2 {
    let yaw = yaw_degrees.to_radians();
    vec2(yaw.cos(), yaw.sin())
}

fn distance_2d(a: Vec2, b: Vec2) -> f32 {
    let delta = a - b;
    (delta.x * delta.x + delta.y * delta.y).sqrt()
}

fn overhead_map(map_name: &str) -> Option<OverheadMap> {
    match map_name.trim_end_matches('\0') {
        "de_mirage" => Some(OverheadMap::new(-3230.0, 1713.0, 5.0)),
        "de_dust2" => Some(OverheadMap::new(-2476.0, 3239.0, 4.4)),
        "de_inferno" => Some(OverheadMap::new(-2087.0, 3870.0, 4.9)),
        "de_overpass" => Some(OverheadMap::new(-4831.0, 1781.0, 5.2)),
        "de_anubis" => Some(OverheadMap::new(-2796.0, 3328.0, 5.22)),
        "de_ancient" => Some(OverheadMap::new(-2953.0, 2164.0, 5.0)),
        "cs_italy" => Some(OverheadMap::new(-2647.0, 2592.0, 4.6)),
        "cs_office" => Some(OverheadMap::new(-1838.0, 1858.0, 4.1)),
        "de_mills" => Some(OverheadMap::new(-4810.0, -320.0, 5.148437)),
        "de_assembly" => Some(OverheadMap::new(1628.0, 4512.0, 2.8)),
        "de_memento" => Some(OverheadMap::new(-2111.6125, 2534.6316, 3.9720395)),
        "de_vertigo" => Some(OverheadMap::new(-3168.0, 1762.0, 4.0)),
        "de_nuke" => Some(OverheadMap::new(-3453.0, 2887.0, 7.0)),
        "de_thera" => Some(OverheadMap::new(-85.609764, 2261.8025, 4.846961)),
        _ => None,
    }
}
