// SPDX-License-Identifier: GPL-3.0-or-later
use super::{Hit, Projectile, ProjectileWorld};
use horizon_core::geometry::Vec3;

pub const PRIMARY: &str = "lunatic:yuuka_danmaku_medium";
pub const FLOWERS: [&str; 3] = [
    "lunatic:yuuka_danmaku_flower_large",
    "lunatic:yuuka_danmaku_flower_medium",
    "lunatic:yuuka_danmaku_flower_small",
];

pub fn ours(p: &Projectile) -> bool {
    p.kind.id() == PRIMARY || FLOWERS.contains(&p.kind.id())
}

pub fn tick(p: &mut Projectile, world: &dyn ProjectileWorld) -> Option<(bool, Hit)> {
    if !ours(p) {
        return None;
    }
    let flower = p.kind.id() != PRIMARY;
    let range = if flower { 20 } else { 16 };
    if p.age >= range {
        return Some((false, Hit::None));
    }
    p.set_old_pos_and_rot();
    let from = p.position;
    let advance = if p.age + 1 == range { 0.65 } else { 1.0 };
    let mut step = p.velocity.scale(advance);
    if flower {
        let side = Vec3::new(p.velocity.z, 0.0, -p.velocity.x).normalize();
        let sway = |tick| 0.30 * (f64::from(tick) * std::f64::consts::TAU / 10.0).sin();
        step = step.add_vector(side.scale(sway(p.age + 1) - sway(p.age)));
    }
    p.age += 1;
    let to = from.add_vector(step);
    let block = world.clip(from, to);
    let end = block.map_or(to, |hit| hit.0);
    let hit = world
        .targets()
        .iter()
        .filter(|(who, _)| Some(*who) != p.owner)
        .filter_map(|(who, bounds)| {
            bounds
                .inflate(Vec3::new(0.18, 0.18, 0.18))
                .clip(from, end)
                .map(|at| (*who, at))
        })
        .min_by(|(_, a), (_, b)| {
            from.distance_squared(*a)
                .total_cmp(&from.distance_squared(*b))
        });
    if let Some((who, at)) = hit {
        p.position = at;
        return Some((false, Hit::Entity { who, at }));
    }
    if let Some((at, pos, face)) = block {
        p.position = at;
        return Some((false, Hit::Block { pos, at, face }));
    }
    p.position = to;
    p.needs_sync = flower;
    Some((true, Hit::None))
}
