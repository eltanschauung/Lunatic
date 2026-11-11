// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_client::{
    mobs::Who,
    projectiles::{
        Projectile, ProjectileKind, gourd_motion as motion, gourd_physics as f, thrown_type,
    },
};
use horizon_core::geometry::Vec3;
use std::{
    cell::RefCell,
    collections::HashMap,
    hash::{Hash, Hasher},
};
pub const ITEM: &str = "lunatic:ibuki_gourd";
const REQUEST: &str = "lunatic:gourd_attack";
const UPDATE: &str = "lunatic:gourd_tethers";
#[derive(Clone)]
struct Attack {
    world: u64,
    dim: String,
    owner: u128,
    id: u32,
    retract: bool,
    exceptional: bool,
}
#[derive(Clone, serde::Serialize, serde::Deserialize)]
struct Tether {
    id: u32,
    owner: u128,
    anchor: [f64; 3],
    phase: String,
}
#[derive(Default)]
struct State {
    attacks: Vec<Attack>,
    ready: HashMap<(u64, u128), i64>,
    views: Vec<Tether>,
}
thread_local! {static STATE:RefCell<State>=RefCell::new(State::default());}
// Client prediction is bounded even between server snapshots.
pub(super) fn predict(
    p: &mut Projectile,
    world: &dyn horizon_client::projectiles::ProjectileWorld,
) {
    let old = p.position;
    let mut next = motion::arr(p.position.add_vector(p.velocity));
    STATE.with_borrow(|s| {
        if let Some(v) = s.views.iter().find(|v| v.id == p.id) {
            let d = f::sub(next, v.anchor);
            let n = f::length(d);
            if n > f::LIMIT {
                next = f::add(v.anchor, f::mul(d, f::LIMIT / n));
            }
        }
    });
    let next = motion::vec(next);
    p.position = world.clip(old, next).map_or(next, |(at, _, _)| at);
}
impl Game<'_> {
    pub(super) fn gourd_input(&mut self) -> bool {
        if !self.native_on("lunatic")
            || self.spectator()
            || self.held().is_none_or(|s| s.item.as_ref() != ITEM)
        {
            return false;
        }
        let mut data = [0; 8];
        data[..4].copy_from_slice(&self.yaw.to_be_bytes());
        data[4..].copy_from_slice(&self.pitch.to_be_bytes());
        let _ = self.native_net_send_to_server(REQUEST, &data);
        self.swing_arm();
        true
    }
    pub(super) fn gourd_request(&mut self, player: u128, channel: &str, data: &[u8]) {
        if channel != REQUEST
            || data.len() != 8
            || self.remote.is_some()
            || !self.native_on("lunatic")
        {
            return;
        }
        let yaw = f32::from_be_bytes(data[..4].try_into().unwrap());
        let pitch = f32::from_be_bytes(data[4..].try_into().unwrap());
        if !yaw.is_finite() || !pitch.is_finite() || pitch.abs() > 90.0 {
            return;
        }
        self.lunatic_as_player(player, |g| g.gourd_click(super::view_vector(yaw, pitch)));
    }
    fn gourd_anchor(&self) -> Option<[f64; 3]> {
        let _p = self.player.as_ref()?;
        let (eye, dir) = self.aim()?;
        Some([eye[0] + dir[2] * 0.32, eye[1] - 0.3, eye[2] - dir[0] * 0.32])
    }
    fn gourd_click(&mut self, dir: [f64; 3]) {
        if self.spectator() || self.held().is_none_or(|s| s.item.as_ref() != ITEM) {
            return;
        }
        let key = (self.lunatic_world_key(), self.native_acting_id());
        let now = self.world.game_time;
        let deployed = STATE.with_borrow_mut(|s| {
            if let Some(a) = s.attacks.iter_mut().find(|a| (a.world, a.owner) == key) {
                a.retract = true;
                true
            } else {
                false
            }
        });
        if deployed {
            return;
        }
        if STATE.with_borrow(|s| s.ready.get(&key).is_some_and(|t| now < *t)) {
            return;
        }
        let Some(anchor) = self.gourd_anchor() else {
            return;
        };
        let Some(t) = thrown_type(motion::ID) else {
            return;
        };
        let mut p = Projectile::new(
            ProjectileKind::Thrown(t),
            motion::vec(f::add(anchor, f::mul(dir, 0.45))),
            Some(Who::Player),
        );
        p.guest = self.lan_acting;
        p.velocity = motion::vec(f::mul(dir, 1.15));
        p.no_gravity = true;
        // add_projectile assigns the real tracking id synchronously.
        self.add_projectile(p);
        let Some(p) = self
            .creatures
            .projectiles
            .iter()
            .filter(|p| motion::ours(p) && p.guest == self.lan_acting)
            .last()
        else {
            return;
        };
        let attack = Attack {
            world: key.0,
            dim: self.dim.id().into(),
            owner: key.1,
            id: p.id,
            retract: false,
            exceptional: false,
        };
        STATE.with_borrow_mut(|s| {
            s.ready.insert(key, now + 80);
            s.attacks.push(attack);
        });
    }
    pub(super) fn gourd_pre(&mut self) {
        if self.remote.is_some() {
            return;
        }
        let world = self.lunatic_world_key();
        let dim = self.dim.id();
        let now = self.world.game_time;
        let live = self
            .creatures
            .projectiles
            .iter()
            .filter(|p| motion::ours(p))
            .map(|p| p.id)
            .collect::<Vec<_>>();
        let mut h = std::collections::hash_map::DefaultHasher::new();
        world.hash(&mut h);
        dim.hash(&mut h);
        motion::begin(h.finish(), &live);
        let mut attacks = STATE.with_borrow_mut(|s| std::mem::take(&mut s.attacks));
        let mut keep = Vec::new();
        for mut a in attacks.drain(..) {
            if a.world != world || a.dim != dim {
                keep.push(a);
                continue;
            }
            let mut anchor = None;
            let mut valid = false;
            self.lunatic_as_player(a.owner, |g| {
                anchor = g.gourd_anchor();
                valid = g.held().is_some_and(|s| s.item.as_ref() == ITEM)
                    && !g.spectator()
                    && g.player.as_ref().is_some_and(|p| !p.health.dead_or_dying());
            });
            let present = live.contains(&a.id);
            if !present && a.retract && !a.exceptional {
                continue;
            }
            if !valid || !present || anchor.is_none() {
                if !a.exceptional {
                    a.exceptional = true;
                    a.retract = true;
                    STATE.with_borrow_mut(|s| {
                        s.ready.insert((world, a.owner), now + 160);
                    });
                }
            }
            if !present {
                continue;
            }
            let p = self
                .creatures
                .projectiles
                .iter()
                .find(|p| p.id == a.id)
                .unwrap();
            // Disconnected/dead/dimension-changed owners cannot retain an orphaned projectile.
            let anchor = anchor.or_else(|| motion::view(a.id).map(|(_, at, _)| at));
            let Some(anchor) = anchor else {
                self.creatures.projectiles.retain(|p| p.id != a.id);
                continue;
            };
            motion::prepare(a.id, a.owner, anchor, p, a.retract);
            keep.push(a);
        }
        STATE.with_borrow_mut(|s| s.attacks = keep);
    }
    pub(super) fn gourd_post(&mut self) {
        if self.remote.is_some() {
            return;
        }
        for e in motion::finish() {
            let horizon_client::projectiles::Hit::Entity { who, at } = e.hit else {
                continue;
            };
            // Existing authoritative projectile damage provides ownership/PvP validation.
            let before = if let Who::Mob(id) = who {
                self.mobs.iter().find(|m| m.id == id).map(|m| m.health)
            } else {
                None
            };
            self.lampad_direct(&e.projectile, who, 6.0, at);
            if let (Who::Mob(id), Some(before)) = (who, before) {
                if self
                    .mobs
                    .iter()
                    .find(|m| m.id == id)
                    .is_none_or(|m| m.health >= before)
                {
                    continue;
                }
            }
            let options = horizon_client::particles::ParticleOptions::simple(
                horizon_client::particles::ParticleType::Explosion,
            );
            self.lan_send_particles(super::lan_particles::QueuedParticle {
                dimension: self.dim.id(),
                options: options.clone(),
                state: None,
                at: motion::arr(at),
                spread: [0.55; 3],
                speed: [0.0; 3],
                count: 8,
                randomization: 0,
            });
            self.particles.add(&options, motion::arr(at), [0.0; 3]);
            self.lunatic_sound("entity.gourd.impact", at, 0.8);
        }
        let views = self
            .creatures
            .projectiles
            .iter()
            .filter(|p| motion::ours(p))
            .filter_map(|p| {
                let (owner, anchor, phase) = motion::view(p.id)?;
                Some(Tether {
                    id: p.id,
                    owner,
                    anchor,
                    phase: format!("{phase:?}"),
                })
            })
            .collect::<Vec<_>>();
        STATE.with_borrow_mut(|s| s.views = views.clone());
        let bytes = serde_json::to_vec(&views).unwrap();
        let players = self
            .lan
            .as_ref()
            .map(|h| {
                h.hub
                    .guests
                    .iter()
                    .filter(|g| g.placed && g.dimension == self.dim.id())
                    .map(|g| g.uuid)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        for player in players {
            let _ = self.native_net_send_to_player(player, UPDATE, &bytes);
        }
    }
    pub(super) fn gourd_receive(&mut self, channel: &str, data: &[u8]) {
        if channel == UPDATE {
            if let Ok(views) = serde_json::from_slice::<Vec<Tether>>(data) {
                STATE.with_borrow_mut(|s| s.views = views);
            }
        }
    }
    fn gourd_view_owner(&self) -> u128 {
        self.remote
            .as_ref()
            .map(|r| r.profile())
            .or(self.lan_acting_uuid)
            .unwrap_or(0)
    }
    pub(super) fn gourd_deployed(&self) -> bool {
        STATE.with_borrow(|s| s.views.iter().any(|v| v.owner == self.gourd_view_owner()))
    }
    pub(super) fn gourd_lines(&self, alpha: f64, lines: &mut Vec<horizon_client::gizmos::Line>) {
        if !self.native_on("lunatic") {
            return;
        }
        STATE.with_borrow(|s| {
            for v in &s.views {
                let Some(p) = self
                    .creatures
                    .projectiles
                    .iter()
                    .find(|p| p.id == v.id && motion::ours(p))
                else {
                    continue;
                };
                let at = motion::arr(p.previous.lerp(p.position, alpha));
                let mut hand = v.anchor;
                if v.owner == self.gourd_view_owner() {
                    if let Some((eye, dir)) = self.aim() {
                        hand = [
                            eye[0] + dir[0] * 0.45 + dir[2] * 0.32,
                            eye[1] - 0.32 + dir[1] * 0.45,
                            eye[2] + dir[2] * 0.45 - dir[0] * 0.32,
                        ];
                    }
                }
                let delta = f::sub(at, hand);
                let distance = f::length(delta);
                let slack = if v.phase == "Retracting" {
                    0.1
                } else {
                    ((8.0 - distance).max(0.0) * 0.18).min(1.2)
                };
                let point = |t: f64| {
                    let mut x = f::add(hand, f::mul(delta, t));
                    x[1] -= 4.0 * slack * t * (1.0 - t);
                    if v.phase == "Grounded" {
                        x[1] = x[1].max(at[1].min(hand[1]));
                    }
                    x
                };
                for i in 0..32 {
                    lines.push(horizon_client::gizmos::Line {
                        start: point(i as f64 / 32.0),
                        end: point((i + 1) as f64 / 32.0),
                        color: 0xFFDBBDA3,
                        width: 1.6,
                    });
                }
            }
        });
    }
    pub(super) fn gourd_status(&self, out: &mut Vec<serde_json::Value>) {
        STATE.with_borrow(|s| {
            for v in &s.views {
                if let Some(o) = out
                    .iter_mut()
                    .find(|o| o["id"].as_u64() == Some(v.id as u64))
                {
                    o["gourd_phase"] = v.phase.clone().into();
                    o["gourd_anchor"] = serde_json::json!(v.anchor);
                }
            }
        });
    }
    pub(super) fn gourd_stop(&mut self) {
        STATE.with_borrow_mut(|s| *s = State::default());
        motion::reset();
    }
}
