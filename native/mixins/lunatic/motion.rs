// SPDX-License-Identifier: GPL-3.0-or-later
// Native flight for Lunatic's thrown entity types. Ordinary projectiles are untouched.
// State is separated by simulation thread and level; the server owns hits and expiry.
use super::{Hit, Projectile, ProjectileKind, ProjectileWorld};
use crate::mobs::Who;
use horizon_core::geometry::Vec3;
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    time::{Duration, Instant},
};

pub const ITEMS: [&str; 3] = [
    "lunatic:yin_yang_orb_precision",
    "lunatic:yin_yang_orb_homing",
    "lunatic:yin_yang_orb_bouncing",
];
pub const LABELS: [&str; 3] = ["Precision", "Homing", "Bouncing"];
pub fn variant(p: &Projectile) -> Option<usize> {
    let ProjectileKind::Thrown(t) = p.kind else {
        return None;
    };
    ITEMS.iter().position(|id| *id == t.id)
}
pub fn choose(current: Option<usize>, counts: [u32; 3], cycle: bool) -> Option<usize> {
    if !cycle {
        if let Some(i) = current.filter(|i| *i < 3 && counts[*i] > 0) {
            return Some(i);
        }
    }
    let start = if cycle {
        current
            .filter(|i| *i < 3 && counts[*i] > 0)
            .map_or(0, |i| (i + 1) % 3)
    } else {
        0
    };
    (0..3).map(|n| (start + n) % 3).find(|i| counts[*i] > 0)
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Sound {
    Bounce,
    Impact,
    CloseDetonation,
}
#[derive(Clone)]
pub struct Event {
    pub sound: Sound,
    pub projectile: Projectile,
    pub position: Vec3,
    pub normal: [i8; 3],
}
struct Flight {
    bounces: u8,
    target: Option<Who>,
    seen: Instant,
}
#[derive(Default)]
struct State {
    level: u64,
    server: bool,
    flights: HashMap<(u64, u32), Flight>,
    events: Vec<Event>,
    live: HashSet<u32>,
}
thread_local! {static STATE:RefCell<State>=RefCell::new(State::default());}
pub fn begin(level: u64, live: &[u32]) {
    STATE.with_borrow_mut(|s| {
        s.level = level;
        s.server = true;
        s.live.clear();
        s.live.extend(live.iter().copied());
        let live = &s.live;
        s.flights.retain(|(l, id), f| {
            if *l == level {
                live.contains(id)
            } else {
                f.seen.elapsed() < Duration::from_secs(120)
            }
        });
    });
}
pub fn finish() -> Vec<Event> {
    STATE.with_borrow_mut(|s| {
        s.server = false;
        std::mem::take(&mut s.events)
    })
}
pub fn reset() {
    STATE.with_borrow_mut(|s| *s = State::default());
}
fn event(s: &mut State, p: &Projectile, sound: Sound) {
    event_normal(s, p, sound, [0, 1, 0]);
}
fn event_normal(s: &mut State, p: &Projectile, sound: Sound, normal: [i8; 3]) {
    if s.server {
        let mut projectile = p.clone();
        // Horizon temporarily substitutes the shooter entity in the collision world.
        if projectile.guest.is_some() {
            projectile.owner = Some(Who::Player);
        }
        s.events.push(Event {
            sound,
            projectile,
            position: p.position,
            normal,
        });
    }
}
fn normalize(v: Vec3) -> Vec3 {
    let l = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt().max(1e-12);
    Vec3::new(v.x / l, v.y / l, v.z / l)
}
fn plus(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.x + b.x, a.y + b.y, a.z + b.z)
}
fn scale(a: Vec3, f: f64) -> Vec3 {
    Vec3::new(a.x * f, a.y * f, a.z * f)
}
fn distance(a: Vec3, b: Vec3) -> f64 {
    (a.x - b.x).powi(2) + (a.y - b.y).powi(2) + (a.z - b.z).powi(2)
}
/// Reuses Horizon's swept block collider and entity boxes. Distances are per 20 Hz tick.
pub fn tick(p: &mut Projectile, w: &dyn ProjectileWorld) -> Option<(bool, Hit)> {
    let kind = variant(p)?;
    Some(STATE.with_borrow_mut(|s| {
        let key = (s.level, p.id);
        if p.age == 0 || !s.flights.contains_key(&key) {
            if s.flights.len() > 4096 {
                s.flights
                    .retain(|_, f| f.seen.elapsed() < Duration::from_secs(120));
            }
            s.flights.insert(
                key,
                Flight {
                    bounces: [5, 2, 4][kind],
                    target: None,
                    seen: Instant::now(),
                },
            );
        }
        let mut flight = s.flights.remove(&key).expect("flight initialized");
        flight.seen = Instant::now();
        p.previous = p.position;
        p.age += 1;
        p.no_gravity = true;
        if p.age >= [120, 120, 1200][kind] || p.position.y < -2048.0 {
            event(s, p, Sound::Impact);
            return (false, Hit::None);
        }
        let from = p.position;
        if kind == 2 {
            p.velocity.y -= 12.0 / 400.0;
        }
        if kind == 1 {
            let dir = normalize(p.velocity);
            let eligible = |who: Who, b: horizon_core::geometry::Aabb| {
                if Some(who) == p.owner {
                    return None;
                }
                let c = b.center();
                let d = distance(from, c);
                if d > 35.0 * 35.0 {
                    return None;
                }
                let to = normalize(Vec3::new(c.x - from.x, c.y - from.y, c.z - from.z));
                let dot = dir.x * to.x + dir.y * to.y + dir.z * to.z;
                if dot < 0.85 {
                    return None;
                }
                if w.clip(from, c).is_some() {
                    return None;
                }
                Some((who, c, d, dot))
            };
            let target = flight
                .target
                .and_then(|id| w.targets().iter().find(|(who, _)| *who == id))
                .and_then(|(who, b)| eligible(*who, *b))
                .or_else(|| {
                    w.targets()
                        .iter()
                        .filter_map(|(who, b)| eligible(*who, *b))
                        .max_by(|a, b| a.3.total_cmp(&b.3))
                });
            if let Some((who, center, d, _)) = target {
                flight.target = Some(who);
                if d <= 1.0 {
                    event(s, p, Sound::CloseDetonation);
                    event(s, p, Sound::Impact);
                    return (false, Hit::None);
                }
                let wanted = normalize(Vec3::new(
                    center.x - from.x,
                    center.y - from.y,
                    center.z - from.z,
                ));
                let blend = 5.5 / 20.0;
                let heading = normalize(plus(scale(dir, 1.0 - blend), scale(wanted, blend)));
                let speed = 1.25 * (0.2 + 0.8 * (f64::from(p.age) / 20.0).min(1.0));
                p.velocity = scale(heading, speed);
                p.needs_sync = true;
            } else {
                flight.target = None;
            }
        }
        let to = plus(from, p.velocity);
        let block = w.clip(from, to);
        let end = block.map_or(to, |b| b.0);
        let mut hit: Option<(Who, Vec3, f64)> = None;
        for (who, b) in w.targets() {
            if Some(*who) == p.owner && !p.left_owner {
                continue;
            }
            if let Some(at) = b.inflate(Vec3::new(0.3, 0.3, 0.3)).clip(from, end) {
                let d = distance(from, at);
                if hit.is_none_or(|h| d < h.2) {
                    hit = Some((*who, at, d));
                }
            }
        }
        if !p.left_owner {
            p.left_owner = w
                .targets()
                .iter()
                .find(|(who, _)| Some(*who) == p.owner)
                .is_none_or(|(_, b)| !b.inflate(Vec3::new(1.0, 1.0, 1.0)).contains(p.position));
        }
        if let Some((who, at, _)) = hit {
            p.position = at;
            if kind == 1
                && flight.target == Some(who)
                && w.targets()
                    .iter()
                    .find(|(id, _)| *id == who)
                    .is_some_and(|(_, b)| distance(at, b.center()) <= 1.0)
            {
                // Swept proximity must survive a full 1.25-block server tick.
                event(s, p, Sound::CloseDetonation);
            }
            event(s, p, Sound::Impact);
            return (false, Hit::Entity { who, at });
        }
        if let Some((at, pos, face)) = block {
            p.position = at;
            if flight.bounces == 0 {
                event_normal(s, p, Sound::Impact, face.map(|v| v as i8));
                return (false, Hit::Block { pos, at, face });
            }
            flight.bounces -= 1;
            let n = Vec3::new(f64::from(face[0]), f64::from(face[1]), f64::from(face[2]));
            let dot = p.velocity.x * n.x + p.velocity.y * n.y + p.velocity.z * n.z;
            p.velocity = plus(p.velocity, scale(n, -2.0 * dot));
            if kind == 2 {
                p.velocity = scale(p.velocity, 0.7);
                p.velocity.y += 2.0 / 20.0;
            }
            p.position = plus(at, scale(n, 0.05));
            p.needs_sync = true;
            event_normal(s, p, Sound::Bounce, face.map(|v| v as i8));
        } else {
            p.position = to;
        }
        // The model's yaw/pitch retain the flight direction; the sphere is three dimensional.
        p.yaw = p.velocity.z.atan2(p.velocity.x).to_degrees() as f32 - 90.0;
        p.pitch = (-p.velocity.y)
            .atan2((p.velocity.x * p.velocity.x + p.velocity.z * p.velocity.z).sqrt())
            .to_degrees() as f32;
        s.flights.insert(key, flight);
        (true, Hit::None)
    }))
}
#[cfg(test)]
mod tests {
    use super::*;
    use horizon_core::geometry::Aabb;
    struct World {
        wall: bool,
        targets: Vec<(Who, Aabb)>,
    }
    impl ProjectileWorld for World {
        fn clip(&self, from: Vec3, to: Vec3) -> Option<(Vec3, [i32; 3], [i32; 3])> {
            (self.wall && to.z >= 1.0 && from.z < 1.0).then_some((
                Vec3::new(to.x, to.y, 1.0),
                [0, 0, 1],
                [0, 0, -1],
            ))
        }
        fn in_water(&self, _: Aabb) -> bool {
            false
        }
        fn targets(&self) -> &[(Who, Aabb)] {
            &self.targets
        }
    }
    fn shot(kind: usize) -> Projectile {
        static TYPES: std::sync::OnceLock<[super::super::ThrownType; 3]> =
            std::sync::OnceLock::new();
        let t = &TYPES.get_or_init(|| {
            std::array::from_fn(|i| super::super::ThrownType {
                id: ITEMS[i],
                item: ITEMS[i],
                damage: 3.0,
            })
        })[kind];
        let mut p = Projectile::new(
            ProjectileKind::Thrown(t),
            Vec3::new(0.0, 2.0, 0.0),
            Some(Who::Player),
        );
        p.id = 1;
        p.velocity = Vec3::new(0.0, 0.0, 1.25);
        p
    }
    #[test]
    fn selection_cycles_only_available_and_falls_back_when_depleted() {
        assert_eq!(choose(None, [0, 0, 0], true), None);
        assert_eq!(choose(None, [1, 0, 1], true), Some(0));
        assert_eq!(choose(Some(0), [1, 0, 1], true), Some(2));
        assert_eq!(choose(Some(2), [1, 0, 1], true), Some(0));
        assert_eq!(choose(Some(1), [2, 0, 1], false), Some(0));
        assert_eq!(choose(Some(1), [2, 0, 1], true), Some(0));
    }
    #[test]
    fn precision_has_no_gravity_bounces_and_expires() {
        reset();
        let w = World {
            wall: true,
            targets: vec![],
        };
        let mut p = shot(0);
        begin(1, &[1]);
        assert_eq!(tick(&mut p, &w), Some((true, Hit::None)));
        assert!(p.velocity.z < 0.0);
        assert_eq!(p.velocity.y, 0.0);
        let e = finish();
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].sound, Sound::Bounce);
        p.age = 119;
        begin(1, &[1]);
        assert_eq!(tick(&mut p, &w), Some((false, Hit::None)));
        assert_eq!(finish()[0].sound, Sound::Impact);
    }
    #[test]
    fn bouncing_uses_gravity_and_damping() {
        reset();
        let w = World {
            wall: true,
            targets: vec![],
        };
        let mut p = shot(2);
        begin(1, &[1]);
        tick(&mut p, &w);
        assert!((p.velocity.z + 0.875).abs() < 1e-9);
        assert!(p.velocity.y > 0.0);
        finish();
    }
    #[test]
    fn entity_sweep_does_not_tunnel() {
        reset();
        let w = World {
            wall: false,
            targets: vec![(
                Who::Mob(4),
                Aabb::new(Vec3::new(-0.2, 1.5, 0.7), Vec3::new(0.2, 2.5, 0.9)),
            )],
        };
        let mut p = shot(0);
        begin(1, &[1]);
        assert!(matches!(
            tick(&mut p, &w),
            Some((
                false,
                Hit::Entity {
                    who: Who::Mob(4),
                    ..
                }
            ))
        ));
        finish();
    }
    #[test]
    fn homing_turns_and_swept_close_hits_emit_both_original_sounds() {
        reset();
        let mut p = shot(1);
        begin(1, &[1]);
        let far = World {
            wall: false,
            targets: vec![(
                Who::Mob(4),
                Aabb::new(Vec3::new(1.9, 1.9, 9.9), Vec3::new(2.1, 2.1, 10.1)),
            )],
        };
        assert_eq!(tick(&mut p, &far), Some((true, Hit::None)));
        assert!(p.velocity.x > 0.0);
        assert!(p.velocity.z < 1.25);
        assert!(finish().is_empty());
        let near = World {
            wall: false,
            targets: vec![(
                Who::Mob(4),
                Aabb::new(Vec3::new(-0.1, 1.9, 1.4), Vec3::new(0.1, 2.1, 1.6)),
            )],
        };
        p.position = Vec3::new(0.0, 2.0, 0.0);
        p.velocity = Vec3::new(0.0, 0.0, 1.25);
        p.age = 20;
        begin(1, &[1]);
        assert!(matches!(
            tick(&mut p, &near),
            Some((false, Hit::Entity { .. }))
        ));
        assert_eq!(
            finish().iter().map(|e| e.sound).collect::<Vec<_>>(),
            vec![Sound::CloseDetonation, Sound::Impact]
        );
    }
    #[test]
    fn ordinary_projectile_is_unchanged() {
        let mut p = Projectile::new(ProjectileKind::Snowball, Vec3::new(0.0, 1.0, 0.0), None);
        assert_eq!(
            tick(
                &mut p,
                &World {
                    wall: false,
                    targets: vec![]
                }
            ),
            None
        );
        assert_eq!(p.age, 0);
    }
}
