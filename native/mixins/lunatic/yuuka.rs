// SPDX-License-Identifier: GPL-3.0-or-later
use super::{Game, lan_particles::QueuedParticle, lunatic_weapons::YUUKA_PARASOL};
use horizon_client::{
    mobs::{Damage, DamageKind, Who},
    particles::{ParticleOptions, ParticleType},
    projectiles::{Projectile, ProjectileKind, lunatic_yuuka as flight, thrown_type},
};
use horizon_core::{geometry::Vec3, random::RandomSource};
use horizon_world::nbt::{Compound, Tag};
use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

const SPELL: &str = "lunatic:sunflower_uprising";
const DESCENT: i64 = 16;
type Field = (u64, &'static str);

#[derive(Default)]
struct State {
    last_fire: HashMap<(Field, u128, bool), i64>,
    volleys: Vec<Volley>,
    pillars: Vec<Pillar>,
}
struct Volley {
    field: Field,
    owner: u128,
    due: i64,
    direction: Vec3,
    item: &'static str,
}
struct Pillar {
    field: Field,
    base: [i32; 3],
    center: Vec3,
    rise: i64,
    due: i64,
    dirt: usize,
    sound: bool,
    breakup: bool,
    bodies: Vec<u32>,
    damaged: Rc<RefCell<HashSet<u32>>>,
}

thread_local! { static STATE: RefCell<State> = RefCell::new(State::default()); }

fn xyz(p: Vec3) -> [f64; 3] {
    [p.x, p.y, p.z]
}

fn pillar_height(age: i64, rise: i64) -> f64 {
    if age < rise {
        let progress = (age + 1) as f64 / rise as f64;
        1.0 - (1.0 - progress).powi(2)
    } else {
        1.0 - (((age - rise) as f64 / DESCENT as f64).min(1.0)).powi(2)
    }
}

impl Game<'_> {
    fn yuuka_field(&self) -> Field {
        (self.lunatic_world_key(), self.dim.id())
    }

    pub(super) fn lunatic_yuuka_fire(&mut self, hand: u8, alternate: bool) {
        if self.remote.is_some()
            || self.spectator()
            || !self.native_on("lunatic")
            || self.lunatic_parasol_hand() != Some(hand)
            || self
                .player
                .as_ref()
                .is_none_or(|p| p.health.dead_or_dying())
        {
            return;
        }
        let Some((eye, aim)) = self.aim() else {
            return;
        };
        let field = self.yuuka_field();
        let owner = self.native_acting_id();
        let now = self.world.game_time;
        let interval = if alternate { 14 } else { 9 };
        let fired = STATE.with_borrow_mut(|s| {
            let last = s
                .last_fire
                .entry((field, owner, alternate))
                .or_insert(now - interval);
            if now >= *last && now - *last < interval {
                return false;
            }
            *last = now;
            true
        });
        if !fired {
            return;
        }

        let forward = Vec3::new(aim[0], aim[1], aim[2]).normalize();
        if alternate {
            let order = if self.world.random.next_double() < 0.5 {
                [1, 2]
            } else {
                [2, 1]
            };
            self.lunatic_yuuka_shot(flight::FLOWERS[0], forward);
            STATE.with_borrow_mut(|s| {
                for (n, index) in order.into_iter().enumerate() {
                    s.volleys.push(Volley {
                        field,
                        owner,
                        due: now + (n as i64 + 1) * 3,
                        direction: forward,
                        item: flight::FLOWERS[index],
                    });
                }
            });
        } else {
            let yaw = f64::from(self.yaw).to_radians();
            let right = Vec3::new(yaw.cos(), 0.0, yaw.sin());
            for angle in [-30.0_f64, -20.0, -10.0, 0.0, 10.0, 20.0, 30.0] {
                let (sin, cos) = angle.to_radians().sin_cos();
                self.lunatic_yuuka_shot(
                    flight::PRIMARY,
                    forward.scale(cos).add_vector(right.scale(sin)).normalize(),
                );
            }
        }
        self.lunatic_action_sound(
            YUUKA_PARASOL.attack(alternate),
            Vec3::new(eye[0], eye[1], eye[2]),
        );
    }

    fn lunatic_yuuka_shot(&mut self, item: &str, direction: Vec3) {
        let (Some(kind), Some(eye)) = (thrown_type(item), self.eye_position()) else {
            return;
        };
        let at = Vec3::new(eye[0], eye[1], eye[2]).add_vector(direction.scale(flight::SHOT_OFFSET));
        let mut shot = Projectile::new(ProjectileKind::Thrown(kind), at, Some(Who::Player));
        shot.velocity = if item == flight::PRIMARY {
            direction.scale(flight::PRIMARY_START_SPEED)
        } else {
            direction
        };
        shot.no_gravity = true;
        shot.guest = self.lan_acting;
        self.add_projectile(shot);
    }

    pub(super) fn lunatic_yuuka_spell(&mut self) {
        if self.spectator()
            || !self.lunatic_has_race("yuuka")
            || self.potions.effects.get(SPELL).is_none()
        {
            return;
        }
        let owner = self.native_acting_id();
        self.native_remove_effect(owner, SPELL);
        let Some(player) = self.player.as_ref().filter(|p| !p.health.dead_or_dying()) else {
            return;
        };
        let center = player.position;
        let [x, y, z] = xyz(center).map(|n| n.floor() as i32);
        let now = self.world.game_time;
        let field = self.yuuka_field();
        let damaged = Rc::new(RefCell::new(HashSet::new()));
        let mut pillars = Vec::new();
        for dx in -4_i32..=4 {
            for dz in -4_i32..=4 {
                let distance = dx * dx + dz * dz;
                if distance == 0 || distance > 20 || (distance > 4 && (dx + dz).rem_euclid(2) != 0)
                {
                    continue;
                }
                let rise = 16
                    + self
                        .world
                        .random
                        .next_int_bound(15)
                        .expect("positive bound") as i64;
                let dirt =
                    2 + self.world.random.next_int_bound(3).expect("positive bound") as usize;
                let sound = self.world.random.next_int_bound(3).expect("positive bound") == 0;
                let breakup = self.world.random.next_int_bound(4).expect("positive bound") == 0;
                pillars.push(Pillar {
                    field,
                    base: [x + dx, y, z + dz],
                    center,
                    rise,
                    dirt,
                    sound,
                    breakup,
                    due: now + 1 + (f64::from(distance).sqrt() * 2.0) as i64,
                    bodies: Vec::new(),
                    damaged: Rc::clone(&damaged),
                });
            }
        }
        STATE.with_borrow_mut(|s| s.pillars.extend(pillars));
    }

    pub(super) fn lunatic_yuuka_tick(&mut self) {
        let field = self.yuuka_field();
        let now = self.world.game_time;
        let (volleys, pillars) = STATE.with_borrow_mut(|s| {
            let (ready, future) = std::mem::take(&mut s.volleys)
                .into_iter()
                .partition(|v| v.field == field && v.due <= now);
            s.volleys = future;
            let (active, other) = std::mem::take(&mut s.pillars)
                .into_iter()
                .partition(|p| p.field == field && p.due <= now);
            s.pillars = other;
            s.last_fire
                .retain(|(key, _, _), at| *key != field || now - *at < 1200);
            (ready, active)
        });
        for volley in volleys {
            let shoot = |g: &mut Game<'_>| {
                if g.yuuka_field() == volley.field
                    && g.lunatic_parasol_hand().is_some()
                    && !g.spectator()
                    && g.player.as_ref().is_some_and(|p| !p.health.dead_or_dying())
                {
                    g.lunatic_yuuka_shot(volley.item, volley.direction);
                }
            };
            if self.native_acting_id() == volley.owner {
                shoot(self);
            } else {
                self.lunatic_as_player(volley.owner, shoot);
            }
        }
        for mut pillar in pillars {
            if self.lunatic_tick_pillar(&mut pillar, now) {
                STATE.with_borrow_mut(|s| s.pillars.push(pillar));
            }
        }
    }

    fn lunatic_tick_pillar(&mut self, p: &mut Pillar, now: i64) -> bool {
        let dimension = self.dim.id();
        let [x, original_y, z] = p.base;
        let age = now - p.due;
        if p.bodies.is_empty() {
            if !self.world.chunks.contains_key(&(x >> 4, z >> 4)) {
                return false;
            }
            let Some(y) = ((original_y - 4)..=(original_y + 3)).rev().find(|&y| {
                let clear = |y| {
                    matches!(
                        self.native_block([x, y, z]).as_str(),
                        "minecraft:air" | "minecraft:cave_air" | "minecraft:void_air"
                    )
                };
                clear(y)
                    && clear(y + 1)
                    && self
                        .native_block_collision([x, y - 1, z])
                        .iter()
                        .any(|aabb| aabb[4] >= 0.99)
            }) else {
                return false;
            };
            p.base[1] = y;
            for i in 0..p.dirt + 3 {
                let block = if i < p.dirt {
                    "minecraft:dirt"
                } else if i == p.dirt {
                    "minecraft:grass_block"
                } else {
                    "minecraft:sunflower"
                };
                let mut state = Tag::compound([("Name", Tag::String(block.into()))]);
                if i > p.dirt
                    && let Tag::Compound(state) = &mut state
                {
                    let half = if i == p.dirt + 1 { "lower" } else { "upper" };
                    state.insert(
                        "Properties".into(),
                        Tag::compound([("half", Tag::String(half.into()))]),
                    );
                }
                let nbt = Compound::from_iter([
                    ("block_state".into(), state),
                    ("teleport_duration".into(), Tag::Int(1)),
                    ("horizon:transient".into(), Tag::Byte(1)),
                ]);
                let at = [
                    f64::from(x),
                    f64::from(y) - (p.dirt + 3 - i) as f64,
                    f64::from(z),
                ];
                if let Some(body) =
                    self.native_summon(dimension, "minecraft:block_display", at, nbt)
                {
                    p.bodies.push(body);
                } else {
                    for id in p.bodies.drain(..) {
                        self.native_remove_entity(dimension, id);
                    }
                    return false;
                }
            }
            let at = Vec3::new(f64::from(x) + 0.5, f64::from(y), f64::from(z) + 0.5);
            self.lunatic_yuuka_break(at, 7);
            if p.sound {
                self.lunatic_sound("entity.gourd.impact", at, 0.55);
            }
        }
        let y = f64::from(p.base[1]);
        if age >= p.rise + DESCENT {
            for id in p.bodies.drain(..) {
                self.native_remove_entity(dimension, id);
            }
            if p.breakup {
                self.lunatic_yuuka_break(Vec3::new(f64::from(x) + 0.5, y, f64::from(z) + 0.5), 13);
            }
            return false;
        }
        let height = (p.dirt + 3) as f64 * pillar_height(age, p.rise);
        for (i, id) in p.bodies.iter().enumerate() {
            let at = [
                f64::from(x),
                y + height - (p.dirt + 3 - i) as f64,
                f64::from(z),
            ];
            self.native_move_entity(dimension, *id, at, 0.0, 0.0);
        }
        if age < p.rise {
            let min = Vec3::new(f64::from(x), y, f64::from(z));
            let max = min.add_vector(Vec3::new(1.0, height, 1.0));
            let mut damaged = p.damaged.borrow_mut();
            for mob in &mut self.mobs {
                if mob.alive()
                    && mob.kind.id != "minecraft:player"
                    && !damaged.contains(&mob.id)
                    && mob.bounds().intersects_corners(min, max)
                    && mob.damage(10.0, Damage::of(DamageKind::Magic))
                {
                    damaged.insert(mob.id);
                    let away = Vec3::new(
                        mob.position.x - p.center.x,
                        0.0,
                        mob.position.z - p.center.z,
                    )
                    .normalize();
                    mob.velocity =
                        mob.velocity
                            .add_vector(Vec3::new(away.x * 0.7, 0.8, away.z * 0.7));
                    mob.needs_sync = true;
                }
            }
        }
        true
    }

    fn lunatic_yuuka_break(&mut self, at: Vec3, count: i32) {
        let dirt = self.pack.assets.registry.default_state("minecraft:dirt");
        let mut options = ParticleOptions::simple(ParticleType::Block);
        options.block = Some(("minecraft:block/dirt".into(), u32::MAX));
        for i in 0..count {
            let angle = f64::from(i) * 2.399963;
            self.particles.add(
                &options,
                [at.x + angle.cos() * 0.3, at.y, at.z + angle.sin() * 0.3],
                [angle.cos() * 0.12, 0.09, angle.sin() * 0.12],
            );
        }
        self.lan_send_particles(QueuedParticle {
            dimension: self.dim.id(),
            options,
            state: dirt,
            at: xyz(at),
            spread: [0.4, 0.1, 0.4],
            speed: [0.12; 3],
            count,
            randomization: 0,
        });
    }

    pub(super) fn lunatic_yuuka_trails(&mut self) {
        let points: Vec<_> = self
            .creatures
            .projectiles
            .iter()
            .filter(|p| p.kind.id() == flight::PRIMARY && p.age % 2 == 0)
            .map(|p| xyz(p.position))
            .collect();
        for at in points {
            let options = ParticleOptions::named("lunatic:yuuka_trail");
            self.particles.add(&options, at, [0.0; 3]);
            self.lan_send_particles(QueuedParticle {
                dimension: self.dim.id(),
                options,
                state: None,
                at,
                spread: [0.0; 3],
                speed: [0.0; 3],
                count: 1,
                randomization: 0,
            });
        }
    }

    pub(super) fn lunatic_yuuka_stop(&mut self) {
        let state = STATE.with_borrow_mut(|s| std::mem::take(&mut *s));
        for pillar in state.pillars {
            for body in pillar.bodies {
                self.native_remove_entity(pillar.field.1, body);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn flower_pillar_rises_then_sinks_at_an_increasing_rate() {
        let rise = 20;
        let height = |age| super::pillar_height(age, rise);
        assert!(height(0) > 0.0);
        assert!(height(15) > height(5));
        assert_eq!(height(rise), 1.0);
        assert!(height(rise + 2) - height(rise + 3) < height(rise + 13) - height(rise + 14));
        assert_eq!(height(rise + super::DESCENT), 0.0);
    }
}
