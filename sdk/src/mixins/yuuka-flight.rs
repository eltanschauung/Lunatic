// SPDX-License-Identifier: GPL-3.0-or-later
use horizon_client_lib::projectiles::{Hit, Projectile, ProjectileWorld};
use horizon_core::geometry::Vec3;

pub const PRIMARY: &str = "lunatic:yuuka_danmaku_medium";
pub const FLOWERS: [&str; 3] = [
    "lunatic:yuuka_danmaku_flower_large",
    "lunatic:yuuka_danmaku_flower_medium",
    "lunatic:yuuka_danmaku_flower_small",
];

pub const PRIMARY_RANGE: u32 = 16;
pub const FLOWER_RANGE: u32 = 20;
pub const SHOT_OFFSET: f64 = 0.35;
pub const PRIMARY_START_SPEED: f64 = 0.4;
const ACCELERATION_TICKS: u32 = 6;

fn primary_speed(age: u32) -> f64 {
    PRIMARY_START_SPEED
        + (1.0 - PRIMARY_START_SPEED) * f64::from(age.min(ACCELERATION_TICKS))
            / f64::from(ACCELERATION_TICKS)
}

fn primary_distance(age: u32) -> f64 {
    let ramp = f64::from(age.min(ACCELERATION_TICKS));
    ramp * (PRIMARY_START_SPEED + primary_speed(age.min(ACCELERATION_TICKS).saturating_sub(1)))
        / 2.0
        + f64::from(age.saturating_sub(ACCELERATION_TICKS))
}

pub fn ours(p: &Projectile) -> bool {
    p.kind.id() == PRIMARY || FLOWERS.contains(&p.kind.id())
}

pub fn tick(p: &mut Projectile, world: &dyn ProjectileWorld) -> Option<(bool, Hit)> {
    if !ours(p) {
        return None;
    }
    let flower = p.kind.id() != PRIMARY;
    let mut step = if flower {
        if p.age >= FLOWER_RANGE {
            return Some((false, Hit::None));
        }
        let advance = if p.age + 1 == FLOWER_RANGE {
            1.0 - SHOT_OFFSET
        } else {
            1.0
        };
        p.velocity.scale(advance)
    } else {
        let remaining = f64::from(PRIMARY_RANGE) - SHOT_OFFSET - primary_distance(p.age);
        if remaining <= 0.0 {
            return Some((false, Hit::None));
        }
        let advance = primary_speed(p.age).min(remaining);
        let velocity = p.velocity.normalize().scale(advance);
        // Send each changed speed to the client. Its remote tick extrapolates
        // the latest speed, so it does not jump ahead at full speed on launch.
        p.needs_sync = p.age <= ACCELERATION_TICKS || advance < 1.0;
        p.velocity = velocity;
        velocity
    };
    p.set_old_pos_and_rot();
    let from = p.position;
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
    p.needs_sync |= flower;
    Some((true, Hit::None))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primary_ramps_for_six_ticks_and_keeps_its_range() {
        assert_eq!(primary_speed(0), PRIMARY_START_SPEED);
        assert_eq!(primary_speed(6), 1.0);
        assert!(primary_speed(5) < 1.0);
        let mut distance = SHOT_OFFSET;
        for age in 0..20 {
            assert!((distance - SHOT_OFFSET - primary_distance(age)).abs() < 1e-9);
            let remaining = f64::from(PRIMARY_RANGE) - distance;
            distance += primary_speed(age).min(remaining);
            if remaining <= primary_speed(age) {
                break;
            }
        }
        assert_eq!(distance, f64::from(PRIMARY_RANGE));
    }
}
