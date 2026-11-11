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
const ITEM_COLORS: [u32; 3] = [0xdc143c, 0x800080, 0x00ff00];
fn ammo_notice(selected: Option<usize>) -> Text {
    match selected {
        Some(i) => Text::literal("Selected ammo: ")
            .append(Text::colored(motion::LABELS[i], ITEM_COLORS[i])),
        None => Text::literal("No Yin-Yang Orbs available."),
    }
}
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
    parasol_updates: Vec<(u128, bool)>,
}
thread_local! {static STATE:RefCell<State>=RefCell::new(State::default());}
fn array(v: Vec3) -> [f64; 3] {
    [v.x, v.y, v.z]
}
fn vec(v: [f64; 3]) -> Vec3 {
    Vec3::new(v[0], v[1], v[2])
}
impl Game<'_> {
    pub(super) fn lunatic_held_name(&self) -> Option<horizon_client::hud_overlays::HeldName> {
        if !self.native_on("lunatic") {
            return None;
        }
        let stack = self.held()?;
        if stack.custom_name().is_some() {
            return None;
        }
        let index = motion::ITEMS
            .iter()
            .position(|id| *id == stack.item.as_ref())?;
        let name = horizon_client::item_tooltip::styled_name(stack, || {
            horizon_client::item_tooltip::name_line(stack, &|key| self.pack.names.display_get(key))
        });
        Some(horizon_client::hud_overlays::HeldName {
            item: stack.item.to_string(),
            hover_name: Text::colored(name.plain(), ITEM_COLORS[index]),
            rarity_color: ITEM_COLORS[index],
            custom_name: false,
        })
    }
    pub(super) fn lunatic_tooltip(&self, stack: &ItemStack, lines: &mut Vec<(Text, u32)>) {
        if !self.native_on("lunatic") {
            return;
        }
        if stack.item.as_ref() == "lunatic:ibuki_gourd" {
            for (a,b) in [
                ("Primary fire for ", ": Hurl the gourd up to 16 blocks away as a flail, dealing great damage. Left click to pull back in. Cooldown: 4s"),
                ("Alt fire: Get drunk. For ", ": Gain super strength for 12-20 seconds before drunkenness sets in, empowering melee damage and providing explosive punches. Effect expiry consumes 2-5 hunger.")
            ] {
                let mut text = Text::colored(a, 0xFFAAAAAA);
                text.spans.extend(Text::colored("Oni", 0xFFFFAA00).spans);
                text.spans.extend(Text::colored(b, 0xFFAAAAAA).spans);
                lines.push((text, u32::MAX));
            }
        }
        if stack.custom_name().is_some() { return; }
        let Some(index) = motion::ITEMS
            .iter()
            .position(|id| *id == stack.item.as_ref())
        else {
            return;
        };
        if let Some((name, base)) = lines.first_mut() {
            *base = u32::MAX;
            for span in &mut name.spans {
                span.style.color = Some(ITEM_COLORS[index]);
            }
        }
    }
    pub(super) fn lunatic_world_key(&self) -> u64 {
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
    pub(super) fn lunatic_parasol_publish(&mut self) {
        let id = self.native_acting_id();
        if let Some(player) = self.player.as_mut() {
            let state = player.__horizon_mixins_lunatic_fairy_mut();
            if state.parasol_sent != Some(state.parasol) {
                state.parasol_sent = Some(state.parasol);
                STATE.with_borrow_mut(|s| s.parasol_updates.push((id, state.parasol)));
            }
        }
    }
    pub(super) fn lunatic_input(&mut self) -> bool {
        if self.gourd_input(){return true;}
        if self.lampad_input(false) { return true; }
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
        if self.native_on("lunatic") && self.held().is_some_and(|s|s.item.as_ref()==super::lunatic_gourd::ITEM){return true;}
        if self.native_on("lunatic") && self.held().is_some_and(|s|s.item.as_ref()=="lunatic:lampad_torch") {return true;}
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
        if !*claimed && stack.item.as_ref() == "lunatic:lampad_torch" {
            *claimed = true;
            // Item use already runs as the authenticated server-side player.
            // Sending another client packet here drops integrated-server uses.
            if let Some((_, dir)) = self.aim() { self.lampad_fire(true, dir); }
            return;
        }
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
        self.lunatic_notice_text(ammo_notice(selected));
    }
    pub(super) fn lunatic_message(&mut self, player: u128, channel: &str, data: &[u8]) {
        self.gourd_request(player,channel,data);
        if channel == super::lunatic_parasol::REQUEST {
            if data.len() == 1 && data[0] <= 1 && self.native_on("lunatic") && self.remote.is_none() {
                self.lunatic_as_player(player, |game| game.lunatic_parasol_toggle(data[0]));
            }
            return;
        }
        if channel != CHANNEL
            || data.len() != 9
            || data[0] > 2
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
        // The click's aim is projectile input, not a change to the player's pose.
        // Mutating player rotation here makes leave_guest send a teleport and clear momentum.
        let direction = super::view_vector(yaw, pitch);
        self.lunatic_as_player(player, |g| if data[0]==0 {g.lunatic_fire(direction)}else{g.lampad_fire(data[0]==2,direction)});
    }

    pub(super) fn lunatic_as_player(&mut self, uuid: u128, run: impl FnOnce(&mut Self)) {
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
                if let Some(swap) = self.enter_guest(&mut host, conn, back) {
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
        self.lunatic_notice_text(Text::literal(message));
    }
    fn lunatic_notice_text(&mut self, message: Text) {
        let player = self.native_acting_id();
        STATE.with_borrow_mut(|s| s.notices.push((player, message)));
    }
    pub(super) fn lunatic_sound(&mut self, event: &str, at: Vec3, gain: f32) {
        let range = if event.ends_with("close_detonation") {
            64.0
        } else if event.ends_with("no_target") || event.starts_with("entity.fairy.") {
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
        for (player, active) in STATE.with_borrow_mut(|s| std::mem::take(&mut s.parasol_updates)) {
            let _ = self.native_net_send_to_player(player, super::lunatic_parasol::STATE, &[u8::from(active)]);
        }
        self.lunatic_flush_bursts();
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

    pub(super) fn lunatic_weapon_ready(&mut self)->bool {
        let key=(self.lunatic_world_key(),self.native_acting_id());
        STATE.with_borrow_mut(|s|{let p=s.preferences.entry(key).or_default();
            if p.last_fire.is_some_and(|t|t.elapsed()<Duration::from_millis(800)){return false;}
            p.last_fire=Some(Instant::now());true})
    }
    fn lunatic_fire(&mut self, direction: [f64; 3]) {
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
        let Some((eye, _)) = self.aim() else {
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
        if !self.creative {
            let held = self.held_mut();
            if let Some(stack) = held {
                stack.damage += 1;
                if stack.damage >= 250 { *held = None; }
            }
        }
        self.lunatic_spawn(kind, Some(direction));
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
                        && valid(Vec3::new(
                            g.position[0],
                            g.position[1] + f64::from(g.dimensions().height) * 0.5,
                            g.position[2],
                        ))
                })
            })
    }
    fn lunatic_spawn(&mut self, kind: usize, direction: Option<[f64; 3]>) {
        let Some(t) = thrown_type(motion::ITEMS[kind]) else {
            return;
        };
        let Some((eye, dir)) = self.aim() else {
            return;
        };
        let dir = direction.unwrap_or(dir);
        let offset = self
            .player
            .as_ref()
            .map_or(0.8, |p| 0.8 * f64::from(p.scale));
        let at = Vec3::new(
            eye[0] + dir[0] * offset,
            eye[1] + dir[1] * offset,
            eye[2] + dir[2] * offset,
        );
        let mut p = Projectile::new(ProjectileKind::Thrown(t), at, Some(Who::Player));
        p.guest = self.lan_acting;
        p.no_gravity = true;
        use horizon_core::random::RandomSource;
        let cross = |a:[f64;3], b:[f64;3]| [a[1]*b[2]-a[2]*b[1],a[2]*b[0]-a[0]*b[2],a[0]*b[1]-a[1]*b[0]];
        let unit = |v:[f64;3]| { let n=v.iter().map(|x|x*x).sum::<f64>().sqrt(); v.map(|x|x/n) };
        let forward=unit(dir);
        let reference=if forward[1].abs()<0.999 { [0.0,1.0,0.0] } else { [1.0,0.0,0.0] };
        let right=unit(cross(forward,reference));
        let up=cross(right,forward);
        // Uniform disk in the aim's transverse plane: no forward/back random offset.
        let radius=[1.0_f64,5.0,2.0][kind].to_radians().tan()*self.world.random.next_double().sqrt();
        let angle=self.world.random.next_double()*std::f64::consts::TAU;
        let spread=unit(std::array::from_fn(|i|forward[i]+radius*(right[i]*angle.cos()+up[i]*angle.sin())));
        p.velocity=Vec3::new(spread[0]*1.25,spread[1]*1.25,spread[2]*1.25);
        self.add_projectile(p);
        self.lunatic_sound("entity.yin_yang_orb.shoot", at, 0.4);
    }
    pub(super) fn lunatic_pre(&mut self) {
        if !self.native_on("lunatic") {
            return;
        }
        self.gourd_pre();
        self.lampad_pre();
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
            self.lunatic_as_player(shot.owner, |g| g.lunatic_spawn(shot.kind, None));
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
        self.gourd_post();
        self.lampad_post();
        for event in motion::finish() {
            self.lunatic_queue_burst(&event);
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
        let _cooldown = horizon_client::health::ProjectileCooldownGuard::new(motion::variant(p).is_some());
        let damage = match p.kind {
            ProjectileKind::Thrown(t) => t.damage,
            _ => return,
        };
        let source = Damage {
            kind: DamageKind::Thrown,
            attacker: p.owner,
            attacker_species: None,
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
        self.gourd_stop();
        self.lampad_stop();
        self.lunatic_clear_bursts();
        STATE.with_borrow_mut(|s| *s = State::default());
        motion::reset();
    }
}
