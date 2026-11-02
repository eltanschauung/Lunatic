// SPDX-License-Identifier: GPL-3.0-or-later
// Server-approved catalog traits and world-persistent starting kits.
use super::{Game, lan::LanHost};
use horizon_client::characters::Character;
use horizon_core::random::RandomSource;
use horizon_world::nbt::Tag;
use serde::{Deserialize, Serialize};

const KIT_KEY: &str = "lunatic:starting_kits";
#[derive(Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Kits {
    awarded: Vec<String>,
    pending: Vec<Stack>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Stack {
    item: String,
    count: u32,
}
impl Game<'_> {
    fn lunatic_character(&self, id: u128) -> Option<&Character> {
        if !self.native_on("lunatic") {
            return None;
        }
        self.character_for(id).or_else(|| {
            (self.remote.is_none() && self.lan_acting.is_none())
                .then_some(self.character.as_ref())
                .flatten()
        })
    }
    pub(super) fn lunatic_character_body(&mut self) {
        if self.player.is_none() {
            return;
        }
        let id = match self.remote.as_ref() {
            Some(remote) => remote.profile(),
            None => self.native_acting_id(),
        };
        let (size, fairy) = self
            .lunatic_character(id)
            .map_or((1.0, false), |c| (c.size, c.has_race("fairy")));
        let potion = self.potions.effects.get("slow_falling").is_some();
        if let Some(p) = self.player.as_mut() {
            p.lunatic_character_body(size, fairy, potion);
        }
    }
    pub(super) fn lunatic_guest_races(&mut self, host: &mut LanHost, conn: u32) {
        let Some(g) = host.hub.guest(conn) else {
            return;
        };
        let jump = g.input & 16 != 0;
        // The input arrives with the first jump's move; remember that it left
        // the ground so the normal jump never consumes the airborne boost.
        let grounded = if g.moves.is_empty() {
            g.on_ground
        } else {
            g.moves_from_ground
        };
        self.as_guest(host, conn, |game| {
            game.lunatic_character_body();
            if let Some(p) = game.player.as_mut() {
                p.lunatic_fairy_step(jump, grounded);
            }
        });
    }
    pub(super) fn lunatic_race_tick(&mut self) {
        if !self.native_on("lunatic") || self.remote.is_some() {
            return;
        }
        self.lunatic_character_body();
        if let Some(event) = self
            .player
            .as_mut()
            .and_then(|p| p.__horizon_mixins_lunatic_fairy_mut().sound.take())
            && let Some(at) = self.eye_position()
        {
            self.lunatic_sound(
                event,
                horizon_core::geometry::Vec3::new(at[0], at[1], at[2]),
                if event.ends_with("jump") { 0.6 } else { 0.55 },
            );
        }
        let id = self.native_acting_id();
        let Some(character) = self.lunatic_character(id) else {
            return;
        };
        let Some(p) = self.player.as_ref() else {
            return;
        };
        if p.health.dead_or_dying()
            || self.spectator()
            || p.__horizon_mixins_lunatic_fairy().kit_checked.as_ref() == Some(&character.id)
        {
            return;
        }
        // Clone only when a kit needs resolving, never on ordinary player ticks.
        let character = character.clone();
        if self
            .level
            .player_extra
            .as_ref()
            .is_some_and(|t| !matches!(t, Tag::Compound(_)))
        {
            self.player
                .as_mut()
                .unwrap()
                .__horizon_mixins_lunatic_fairy_mut()
                .kit_checked = Some(character.id);
            return;
        }
        // Stored in the existing player NBT, saved together with inventory.
        // Unknown/malformed data is preserved and never treated as a fresh kit.
        let raw = self
            .level
            .player_extra
            .as_ref()
            .and_then(|t| t.get(KIT_KEY));
        let mut kits: Kits = match raw {
            None => Kits::default(),
            Some(Tag::String(s)) => match s
                .to_string()
                .ok()
                .and_then(|text| serde_json::from_str(&text).ok())
            {
                Some(kits) => kits,
                None => {
                    self.player
                        .as_mut()
                        .unwrap()
                        .__horizon_mixins_lunatic_fairy_mut()
                        .kit_checked = Some(character.id);
                    return;
                }
            },
            Some(_) => {
                self.player
                    .as_mut()
                    .unwrap()
                    .__horizon_mixins_lunatic_fairy_mut()
                    .kit_checked = Some(character.id);
                return;
            }
        };
        let mut changed = false;
        if !kits.awarded.contains(&character.id) {
            for entry in &character.starting_items {
                let choices: Vec<_> = if entry.choices.is_empty() {
                    vec![entry.item.as_str()]
                } else {
                    entry.choices.iter().map(String::as_str).collect()
                };
                let choices: Vec<_> = choices
                    .into_iter()
                    .filter(|item| self.items().get(item).is_some())
                    .collect();
                if choices.is_empty() {
                    continue;
                }
                let min = entry.min_count.unwrap_or(entry.count).clamp(1, 1024);
                let max = entry.max_count.unwrap_or(entry.count).clamp(min, 1024);
                for _ in 0..entry.rolls.min(64) {
                    let index = self
                        .world
                        .random
                        .next_int_bound(choices.len() as i32)
                        .unwrap_or(0) as usize;
                    let count = min
                        + self
                            .world
                            .random
                            .next_int_bound((max - min + 1) as i32)
                            .unwrap_or(0) as u32;
                    kits.pending.push(Stack {
                        item: choices[index].to_owned(),
                        count,
                    });
                }
            }
            kits.awarded.push(character.id.clone());
            changed = true;
        }
        for stack in &mut kits.pending {
            // Removed items are skipped, including an item removed since the kit was rolled.
            if self.items().get(&stack.item).is_none() {
                stack.count = 0;
                changed = true;
                continue;
            }
            let gave = self.native_give_items(&stack.item, stack.count);
            stack.count -= gave;
            changed |= gave > 0;
        }
        kits.pending.retain(|s| s.count > 0);
        if changed {
            if self.level.player_extra.is_none() {
                self.level.player_extra = Some(Tag::Compound(Default::default()));
            }
            if let Some(Tag::Compound(map)) = &mut self.level.player_extra {
                map.insert(
                    KIT_KEY.into(),
                    Tag::String(
                        serde_json::to_string(&kits)
                            .expect("kit serializes")
                            .as_str()
                            .into(),
                    ),
                );
            }
        }
        if kits.pending.is_empty() {
            self.player
                .as_mut()
                .unwrap()
                .__horizon_mixins_lunatic_fairy_mut()
                .kit_checked = Some(character.id);
        }
    }
}
