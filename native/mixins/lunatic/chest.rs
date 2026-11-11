// Adapted from Female Gender Mod 5.0.0 by pupnewfster (GenderLayer,
// WildfireModelRenderer and default UVs). SPDX-License-Identifier: LGPL-3.0-or-later
// Skin-only port: no armor models, settings, networking or vanilla-player changes.
use super::{
    Atlas, Mob, Part, Vertex,
    anim::{self, Poses},
};
use crate::mobs::{
    Polygon,
    lunatic_chest_physics::{self as physics, Input},
};
use glam::{Mat4, Vec3};

fn half(left: bool, overlay: bool) -> Part {
    let x = if left { -4.0 } else { 0.0 };
    let [a, b, c, d, e, f, g, h] = [
        [x + 4.0, 0.0, 0.0],
        [x + 4.0, 5.0, 0.0],
        [x, 5.0, 0.0],
        [x, 0.0, 3.0],
        [x + 4.0, 0.0, 3.0],
        [x + 4.0, 5.0, 3.0],
        [x, 5.0, 3.0],
        [x, 0.0, 0.0],
    ];
    let faces = [
        [e, a, b, f],
        [h, d, g, c],
        [e, d, h, a],
        [b, c, g, f],
        [a, h, c, b],
    ];
    let uvs = match (left, overlay) {
        (true, false) => [
            [24, 21, 27, 26],
            [16, 21, 20, 26],
            [20, 17, 24, 21],
            [20, 25, 24, 27],
            [20, 21, 24, 26],
        ],
        (false, false) => [
            [28, 21, 32, 26],
            [21, 21, 24, 26],
            [24, 17, 28, 21],
            [24, 25, 28, 27],
            [24, 21, 28, 26],
        ],
        (true, true) => [
            [0, 0, 0, 0],
            [17, 37, 20, 42],
            [20, 34, 24, 37],
            [20, 41, 24, 44],
            [20, 37, 24, 42],
        ],
        (false, true) => [
            [28, 37, 31, 42],
            [0, 0, 0, 0],
            [24, 34, 28, 37],
            [24, 41, 28, 44],
            [24, 37, 28, 42],
        ],
    };
    let normals = [
        [1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, -1.0],
    ];
    let polygons = (0..5)
        .filter_map(|i| {
            let [u0, v0, u1, v1] = uvs[i];
            if u0 == 0 && v0 == 0 && u1 == 0 && v1 == 0 {
                return None;
            }
            let tex = [[u1, v0], [u0, v0], [u0, v1], [u1, v1]];
            Some(Polygon {
                normal: normals[i],
                cull_back: false,
                vertices: std::array::from_fn(|n| {
                    [
                        faces[i][n][0],
                        faces[i][n][1],
                        faces[i][n][2],
                        tex[n][0] as f32 / 64.0,
                        tex[n][1] as f32 / 64.0,
                    ]
                }),
            })
        })
        .collect();
    Part {
        name: "lunatic_chest".into(),
        pose: [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0],
        polygons,
        children: Vec::new(),
    }
}
thread_local! {
    // Entity lifetime IDs survive render copies and never alias a later entity.
    static MOTION: std::cell::RefCell<std::collections::HashMap<u64, (std::time::Instant, physics::Physics)>> = Default::default();
    static HALVES: [Part;4] = [half(true,false),half(false,false),half(true,true),half(false,true)];
}
// Run before the base renderer takes its resource locks. Reuse the same
// posed body, including custom animations and skin scale, for the skin layer.
pub fn render_mob(
    m: &Mob,
    alpha: f32,
    base: Mat4,
    atlas: Atlas<'_>,
    light: u32,
    data: &crate::mobs::MobData,
    visible: bool,
    out: &mut Vec<Vertex>,
) {
    if physics::bust(m.character_chest, m.slim) <= 0.0 || !visible || m.extra.chest.is_some() {
        return;
    }
    let (model, poses, _) = super::posed_body(m, alpha, data);
    let (_, skin) = super::body(m);
    render(
        m, alpha, base, atlas, light, &model, &poses, &skin, visible, out,
    );
}
pub fn render(
    m: &Mob,
    alpha: f32,
    base: Mat4,
    atlas: Atlas<'_>,
    light: u32,
    model: &Part,
    poses: &Poses,
    skin: &str,
    visible: bool,
    out: &mut Vec<Vertex>,
) {
    let bsize = physics::bust(m.character_chest, m.slim);
    if bsize <= 0.0
        || !visible
        || m.kind.id.as_str() != "minecraft:player"
        || m.extra.spectator
        || m.extra.chest.is_some()
    {
        return;
    }
    let Some(uv) = atlas(skin) else {
        return;
    };
    let Some(body) = super::part_matrix(model, base, poses, "body", 0.0) else {
        return;
    };
    let input = Input {
        tick: m.age,
        position: [m.position.x, m.position.y, m.position.z],
        yaw: m.body_yaw,
        vertical_velocity: m.velocity.y as f32,
        speed_squared: m.velocity.length_squared() as f32,
        walk: anim::walk_position(m, 1.0),
        walk_speed: m.limb_speed,
        swing: m.anim.attack_anim,
        crouching: m.extra.crouching,
    };
    let movement = if physics::motion_enabled(m.character_chest) {
        MOTION.with_borrow_mut(|states| {
            let key = m.join.get();
            if states.len() >= 512 && !states.contains_key(&key) {
                if let Some(oldest) = states
                    .iter()
                    .min_by_key(|(_, (at, _))| *at)
                    .map(|(id, _)| *id)
                {
                    states.remove(&oldest);
                }
            }
            let (at, state) = states
                .entry(key)
                .or_insert_with(|| (std::time::Instant::now(), physics::Physics::default()));
            *at = std::time::Instant::now();
            state.sample(input, m.character_chest, alpha)
        })
    } else {
        [0.0; 3]
    };
    let enabled = physics::motion_enabled(m.character_chest);
    let breath = if enabled {
        (-(m.age as f32 + alpha) * 0.09).cos() * -0.045 + 0.045
    } else {
        0.0
    };
    let transform = body * shape(m.character_chest, movement, breath);
    HALVES.with(|halves| {
        for (i, part) in halves.iter().enumerate() {
            if i >= 2 && m.extra.hidden_model_parts & (1 << 1) != 0 {
                continue;
            }
            let jacket = if i >= 2 {
                Mat4::from_translation(Vec3::new(0.0, 0.0, -0.015))
                    * Mat4::from_scale(Vec3::splat(1.05))
            } else {
                Mat4::IDENTITY
            };
            super::draw(
                part,
                transform * jacket,
                poses,
                uv,
                super::body_color(m, alpha),
                light,
                super::body_overlay(m, alpha),
                out,
            );
        }
    });
}

fn shape(chest: f32, movement: [f32; 3], breath: f32) -> Mat4 {
    let bsize = physics::bust(chest, true);
    let enabled = physics::motion_enabled(chest);
    let mut size = (bsize * 1.5).min(0.7);
    if bsize > 0.7 {
        size = bsize;
    }
    size += (bsize - 0.7).abs();
    let z = 0.0625 - bsize * 0.0625 - 0.125;
    let tilt = (size - movement[1] / 12.0).min(size + 0.2).min(1.0);
    // Linear projection from the torso plane; 2 matches the source maximum.
    let projection = Mat4::from_translation(Vec3::new(0.0, 0.0, -0.125))
        * Mat4::from_scale(Vec3::new(1.0, 1.0, (chest - 1.0).clamp(0.0, 1.0)))
        * Mat4::from_translation(Vec3::new(0.0, 0.0, 0.125));
    projection
        * Mat4::from_translation(Vec3::new(
            movement[0] / 32.0,
            0.05625 + movement[1] / 32.0 - if enabled { 0.035 * size } else { 0.0 },
            z,
        ))
        * Mat4::from_rotation_y(movement[2].to_radians())
        * Mat4::from_rotation_x((-35.0 * tilt + breath).to_radians())
        * Mat4::from_scale(Vec3::new(0.9995, 1.0, 1.0))
}
/// The selection screen uses the same skin geometry, resting on its body bone.
pub fn presentation(chest: f32, slim: bool, model: &Part) -> Option<Part> {
    if physics::bust(chest, slim) <= 0.0 {
        return None;
    }
    let mut model = model.clone();
    fn attach(model: &mut Part, chest: f32) -> bool {
        if model.name == "body" {
            let rest = if physics::motion_enabled(chest) {
                [0.0, physics::bust(chest, true) * 1.25, 0.0]
            } else {
                [0.0; 3]
            };
            HALVES.with(|halves| {
                for (i, part) in halves.iter().enumerate() {
                    let mut part = part.clone();
                    let jacket = if i >= 2 {
                        Mat4::from_translation(Vec3::new(0.0, 0.0, -0.015))
                            * Mat4::from_scale(Vec3::splat(1.05))
                    } else {
                        Mat4::IDENTITY
                    };
                    let matrix = shape(chest, rest, 0.0) * jacket;
                    let normals = glam::Mat3::from_mat4(matrix).inverse().transpose();
                    for p in &mut part.polygons {
                        p.normal = (normals * Vec3::from_array(p.normal))
                            .normalize_or_zero()
                            .to_array();
                        for v in &mut p.vertices {
                            let at =
                                matrix.transform_point3(Vec3::new(v[0], v[1], v[2]) / 16.0) * 16.0;
                            v[0] = at.x;
                            v[1] = at.y;
                            v[2] = at.z;
                        }
                    }
                    model.children.push(part);
                }
            });
            return true;
        }
        model.children.iter_mut().any(|p| attach(p, chest))
    }
    attach(&mut model, chest).then_some(model)
}
