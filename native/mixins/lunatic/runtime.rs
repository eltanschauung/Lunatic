// SPDX-License-Identifier: GPL-3.0-or-later
// Server-owned ammo selection, consumption, volley scheduling, sounds and area effects.
use super::{Dim, Game};
use horizon_client::{
    items::ItemStack,
    mobs::{Damage, DamageKind, Who},
    projectiles::{Hit, Projectile, ProjectileKind, lunatic_motion as motion, thrown_type},
    text::Text,
};
use horizon_core::geometry::Vec3;
use std::{
    cell::RefCell,
    collections::HashMap,
    hash::{Hash, Hasher},
    time::{Duration, Instant},
};
const ROD: &str = "lunatic:purification_rod";
const CHANNEL: &str = "lunatic:rod_fire";
#[derive(Default)]
struct Preference {
    selected: Option<usize>,
    last_fire: Option<Instant>,
}
struct QueuedSound {
    dimension: &'static str,
    event: String,
    at: Vec3,
    gain: f32,
    range: f32,
}
struct Shot {
    world: u64,
    dimension: String,
    owner: u128,
    kind: usize,
    due: i64,
}
#[derive(Default)]
struct State {
    preferences: HashMap<(u64, u128), Preference>,
    shots: Vec<Shot>,
    sounds: Vec<QueuedSound>,
    notices: Vec<(u128, Text)>,
}
thread_local! {static STATE:RefCell<State>=RefCell::new(State::default());}
fn array(v: Vec3) -> [f64; 3] {
    [v.x, v.y, v.z]
}
fn vec(v: [f64; 3]) -> Vec3 {
    Vec3::new(v[0], v[1], v[2])
}
impl Game<'_> {
    pub(super) fn lunatic_tooltip(&self, stack: &ItemStack, lines: &mut Vec<(Text, u32)>) {
        if !self.native_on("lunatic") || stack.custom_name().is_some() {
            return;
        }
        let Some(index) = motion::ITEMS
            .iter()
            .position(|id| *id == stack.item.as_ref())
        else {
            return;
        };
        if let Some((name, base)) = lines.first_mut() {
            *base = u32::MAX;
            for span in &mut name.spans {
                span.style.color = Some([0xdc143c, 0x800080, 0x00ff00][index]);
            }
        }
    }
    fn lunatic_world_key(&self) -> u64 {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        self.native_world_dir().hash(&mut h);
        h.finish()
    }
    fn lunatic_counts(&self) -> [u32; 3] {
        let mut counts = [0; 3];
        for stack in self.inventory.iter().flatten() {
            if let Some(i) = motion::ITEMS
                .iter()
                .position(|id| *id == stack.item.as_ref())
            {
                counts[i] += stack.count;
            }
        }
        counts
    }
    pub(super) fn lunatic_input(&mut self) -> bool {
        if !self.native_on("lunatic")
            || self.spectator()
            || self.held().is_none_or(|s| s.item.as_ref() != ROD)
        {
            return false;
        }
        let has_ammo = self.lunatic_counts().iter().any(|n| *n > 0);
        let mut request = [0u8; 9];
        request[1..5].copy_from_slice(&self.yaw.to_be_bytes());
        request[5..9].copy_from_slice(&self.pitch.to_be_bytes());
        if self.native_net_send_to_server(CHANNEL, &request).is_err() {
            return false;
        }
        if has_ammo {
            self.swing_arm();
            self.reset_attack_strength();
        }
        // Without ammo the ordinary stick attack/mining and arm swing still run.
        has_ammo
    }
    pub(super) fn lunatic_continue(&self) -> bool {
        self.native_on("lunatic")
            && self.held().is_some_and(|s| s.item.as_ref() == ROD)
            && self.lunatic_counts().iter().any(|n| *n > 0)
    }
    pub(super) fn lunatic_cycle(&mut self, stack: &ItemStack, claimed: &mut bool) {
        if !self.native_on("lunatic") {
            return;
        }
        if motion::ITEMS.contains(&stack.item.as_ref()) {
            *claimed = true;
            return;
        } // Ammunition does not throw on right click.
        if stack.item.as_ref() != ROD || *claimed || self.spectator() {
            return;
        }
        *claimed = true;
        let key = (self.lunatic_world_key(), self.native_acting_id());
        let counts = self.lunatic_counts();
        let selected = STATE.with_borrow_mut(|s| {
            let p = s.preferences.entry(key).or_default();
            let selected = motion::choose(p.selected, counts, true);
            if selected.is_some() {
                p.selected = selected;
            }
            selected
        });
        let message = selected.map_or_else(
            || "No Yin-Yang Orbs available.".to_owned(),
            |i| format!("Selected ammo: {}", motion::LABELS[i]),
        );
        self.lunatic_notice(&message);
    }
    pub(super) fn lunatic_message(&mut self, player: u128, channel: &str, data: &[u8]) {
        if channel != CHANNEL
            || data.len() != 9
            || data[0] != 0
            || !self.native_on("lunatic")
            || self.remote.is_some()
        {
            return;
        }
        let yaw = f32::from_be_bytes(data[1..5].try_into().expect("checked fire packet"));
        let pitch = f32::from_be_bytes(data[5..9].try_into().expect("checked fire packet"));
        if !yaw.is_finite() || !pitch.is_finite() || !(-90.0..=90.0).contains(&pitch) {
            return;
        }
        let yaw = (yaw + 180.0).rem_euclid(360.0) - 180.0;
        self.lunatic_as_player(player, |g| {
            // Aim belongs to this click, even before the next movement packet arrives.
            g.yaw = yaw;
            g.pitch = pitch;
            if let Some(p) = &mut g.player {
                p.yaw = yaw;
                p.pitch = pitch;
            }
            g.lunatic_fire();
        });
    }

    fn lunatic_as_player(&mut self, uuid: u128, run: impl FnOnce(&mut Self)) {
        let Some(mut host) = self.lan.take() else {
            return;
        };
        let guest = host
            .hub
            .guests
            .iter()
            .find(|g| g.placed && g.uuid == uuid && !g.dead && !g.spectator())
            .map(|g| (g.conn, g.dimension.clone()));
        if let Some((conn, dimension)) = guest {
            let back = self.dim;
            let target = Dim::from_id(&dimension);
            if target == back || self.enter_level(Some(&mut host), target).is_ok() {
                if let Some(swap) = self.enter_guest(&mut host, conn) {
                    // Keep the hub visible for targeting and nested damage helpers.
                    self.lan = Some(host);
                    let outer = self.chat_capture.replace(Vec::new());
                    run(self);
                    let lines =
                        std::mem::replace(&mut self.chat_capture, outer).unwrap_or_default();
                    host = self.lan.take().expect("Lunatic preserves the server hub");
                    self.leave_guest(&mut host, swap);
                    for line in lines {
                        host.hub.send_to(
                            conn,
                            &horizon_protocol::packets::play::Clientbound::SystemChat(
                                horizon_protocol::packets::play::SystemChat {
                                    content: horizon_client::lan::data::component_nbt(
                                        &line.component(),
                                    ),
                                    overlay: false,
                                },
                            ),
                        );
                    }
                }
                if self.dim != back {
                    let _ = self.enter_level(Some(&mut host), back);
                }
            }
        }
        self.lan = Some(host);
    }
    fn lunatic_notice(&mut self, message: &str) {
        let player = self.native_acting_id();
        STATE.with_borrow_mut(|s| s.notices.push((player, Text::literal(message))));
    }
    fn lunatic_sound(&mut self, event: &str, at: Vec3, gain: f32) {
        let range = if event.ends_with("close_detonation") {
            64.0
        } else if event.ends_with("no_target") {
            24.0
        } else {
            32.0
        };
        let dimension = self.dim.id();
        STATE.with_borrow_mut(|s| {
            s.sounds.push(QueuedSound {
                dimension,
                event: format!("lunatic:{event}"),
                at,
                gain,
                range,
            })
        });
    }
    pub(super) fn lunatic_flush_sounds(&mut self) {
        if self.lan.is_none() {
            return;
        }
        for (player, line) in STATE.with_borrow_mut(|s| std::mem::take(&mut s.notices)) {
            self.native_tell(player, line);
        }
        let host = self.lan.as_ref().expect("hub checked");
        let sounds = STATE.with_borrow_mut(|s| std::mem::take(&mut s.sounds));
        for sound in sounds {
            let packet = horizon_protocol::packets::play::Clientbound::Sound(
                horizon_protocol::packets::play::Sound {
                    sound: (-1, Some((sound.event, Some(sound.range)))),
                    source: 7,
                    x: (sound.at.x * 8.0) as i32,
                    y: (sound.at.y * 8.0) as i32,
                    z: (sound.at.z * 8.0) as i32,
                    volume: sound.gain,
                    pitch: 1.0,
                    seed: host.hub.next_sound_seed(),
                },
            );
            for g in host
                .hub
                .guests
                .iter()
                .filter(|g| g.placed && g.dimension == sound.dimension)
            {
                if vec(g.position).distance_squared(sound.at) <= f64::from(sound.range).powi(2) {
                    host.hub.send_to(g.conn, &packet);
                }
            }
        }
    }

    fn lunatic_fire(&mut self) {
        if self.held().is_none_or(|s| s.item.as_ref() != ROD) || self.spectator() {
            return;
        }
        let owner = self.native_acting_id();
        let world = self.lunatic_world_key();
        let counts = self.lunatic_counts();
        let kind = STATE.with_borrow_mut(|s| {
            let pref = s.preferences.entry((world, owner)).or_default();
            if pref
                .last_fire
                .is_some_and(|t| t.elapsed() < Duration::from_millis(800))
            {
                return Err(());
            }
            pref.last_fire = Some(Instant::now());
            let selected = motion::choose(pref.selected, counts, false);
            if selected.is_some() {
                pref.selected = selected;
            }
            Ok(selected)
        });
        let Ok(kind) = kind else {
            return;
        };
        let Some(kind) = kind else {
            self.lunatic_notice("No Yin-Yang orbs available.");
            return;
        };
        let Some((eye, direction)) = self.aim() else {
            return;
        };
        if !self.creative {
            for slot in &mut self.inventory {
                if let Some(stack) = slot
                    .as_mut()
                    .filter(|s| s.item.as_ref() == motion::ITEMS[kind])
                {
                    stack.count -= 1;
                    if stack.count == 0 {
                        *slot = None;
                    }
                    break;
                }
            }
        }
        if kind == 1 && !self.lunatic_has_target(vec(eye), vec(direction)) {
            self.lunatic_sound("item.purification_rod.no_target", vec(eye), 0.6);
            return;
        }
        self.lunatic_sound("item.purification_rod.activate", vec(eye), 0.6);
        self.lunatic_spawn(kind);
        let now = self.world.game_time;
        let dimension = self.dim.id().to_owned();
        STATE.with_borrow_mut(|s| {
            for delay in [3, 6] {
                s.shots.push(Shot {
                    world,
                    dimension: dimension.clone(),
                    owner,
                    kind,
                    due: now + delay,
                });
            }
        });
    }
    fn lunatic_has_target(&self, eye: Vec3, dir: Vec3) -> bool {
        let valid = |at: Vec3| {
            let d = Vec3::new(at.x - eye.x, at.y - eye.y, at.z - eye.z);
            let len = (d.x * d.x + d.y * d.y + d.z * d.z).sqrt();
            len > 0.0
                && len <= 35.0
                && (d.x * dir.x + d.y * dir.y + d.z * dir.z) / len >= 0.85
                && self
                    .clip_ray(array(eye), [d.x / len, d.y / len, d.z / len], len)
                    .is_none()
        };
        self.mobs
            .iter()
            .any(|m| m.alive() && valid(m.bounds().center()))
            || self.lan.as_ref().is_some_and(|h| {
                h.hub.guests.iter().any(|g| {
                    g.placed
                        && !g.dead
                        && !g.spectator()
                        && Some(g.conn) != self.lan_acting
                        && g.dimension == self.dim.id()
                        && valid(Vec3::new(g.position[0], g.position[1] + 0.9, g.position[2]))
                })
            })
    }
    fn lunatic_spawn(&mut self, kind: usize) {
        let Some(t) = thrown_type(motion::ITEMS[kind]) else {
            return;
        };
        let Some((eye, dir)) = self.aim() else {
            return;
        };
        let at = Vec3::new(
            eye[0] + dir[0] * 0.8,
            eye[1] + dir[1] * 0.8,
            eye[2] + dir[2] * 0.8,
        );
        let mut p = Projectile::new(ProjectileKind::Thrown(t), at, Some(Who::Player));
        p.guest = self.lan_acting;
        p.no_gravity = true;
        p.velocity = Vec3::new(dir[0] * 1.25, dir[1] * 1.25, dir[2] * 1.25);
        self.add_projectile(p);
        self.lunatic_sound("entity.yin_yang_orb.shoot", at, 0.4);
    }
    pub(super) fn lunatic_pre(&mut self) {
        if !self.native_on("lunatic") {
            return;
        }
        let world = self.lunatic_world_key();
        let dim = self.dim.id();
        let now = self.world.game_time;
        let shots = STATE.with_borrow_mut(|s| {
            let all = std::mem::take(&mut s.shots);
            let (due, future) = all
                .into_iter()
                .partition::<Vec<_>, _>(|p| p.world == world && p.dimension == dim && p.due <= now);
            s.shots = future;
            due
        });
        for shot in shots {
            self.lunatic_as_player(shot.owner, |g| g.lunatic_spawn(shot.kind));
        }
        let mut h = std::collections::hash_map::DefaultHasher::new();
        world.hash(&mut h);
        dim.hash(&mut h);
        motion::begin(
            h.finish(),
            &self
                .creatures
                .projectiles
                .iter()
                .filter(|p| motion::variant(p).is_some())
                .map(|p| p.id)
                .collect::<Vec<_>>(),
        );
    }
    pub(super) fn lunatic_post(&mut self) {
        for event in motion::finish() {
            let (name, gain) = match event.sound {
                motion::Sound::Bounce => ("entity.yin_yang_orb.bounce", 0.4),
                motion::Sound::Impact => ("entity.yin_yang_orb.impact", 0.9),
                motion::Sound::CloseDetonation => ("entity.yin_yang_orb.close_detonation", 1.0),
            };
            self.lunatic_sound(name, event.position, gain);
            if event.sound == motion::Sound::Impact {
                self.lunatic_area_damage(&event.projectile, event.position);
            }
        }
    }
    fn lunatic_area_damage(&mut self, p: &Projectile, at: Vec3) {
        let damage = match p.kind {
            ProjectileKind::Thrown(t) => t.damage,
            _ => return,
        };
        let source = Damage {
            kind: DamageKind::Thrown,
            attacker: p.owner,
            from: Some(array(p.previous)),
        };
        for m in &mut self.mobs {
            if m.alive() && m.bounds().center().distance_squared(at) <= 9.0 {
                m.damage(damage, source);
            }
        }
        let guests = self.lan_projectile_targets();
        for (who, bounds) in guests {
            if bounds.center().distance_squared(at) > 9.0 {
                continue;
            }
            let hit = Hit::Entity { who, at };
            if let Some(victim) = self.projectile_pvp_victim(p, hit) {
                let _ = self.projectile_hits_player(p, hit, victim);
            }
        }
    }
    pub(super) fn lunatic_stop(&mut self) {
        STATE.with_borrow_mut(|s| *s = State::default());
        motion::reset();
    }
}
