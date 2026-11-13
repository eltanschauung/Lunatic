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
    #[serde(default)]
    initialized: bool,
    #[serde(default)]
    character: Option<String>,
    #[serde(default)]
    deaths: Option<i32>,
    awarded: Vec<String>,
    pending: Vec<Stack>,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Stack {
    item: String,
    count: u32,
    #[serde(default)]
    enchantments: std::collections::BTreeMap<String, u32>,
}
impl Game<'_> {
    pub(super) fn lunatic_has_race(&self, race: &str) -> bool {
        if !self.native_on("lunatic") {
            return false;
        }
        let id = self
            .lan_acting_uuid
            .or_else(|| self.remote.as_ref().map(|remote| remote.profile()));
        let character = match id {
            Some(uuid) => self.character_for(uuid),
            None => self.character.as_ref(),
        };
        character.is_some_and(|c| c.has_race(race))
    }
    pub(super) fn lunatic_crow_tengu(&self) -> bool {
        self.lunatic_has_race("crow_tengu")
    }
    pub(super) fn lunatic_racial_glider(&self) -> bool {
        self.lunatic_crow_tengu() || self.lunatic_has_race("devil")
    }
    pub(super) fn lunatic_fan_boost(&mut self, stack: &crate::items::ItemStack) -> bool {
        const FAN: &str = "lunatic:hauchiwa_fan";
        if stack.item.as_ref() != FAN
            || self.remote.is_some()
            || !self.lunatic_has_race("tengu")
            || self.spectator()
            || !self.player.as_ref().is_some_and(|p| {
                p.fall_flying
                    && !p.__horizon_mixins_lunatic_fairy().racial_glide_armor_blocked
                    && !p.health.dead_or_dying()
            })
            || self.cooldowns.is_on_cooldown(FAN)
        {
            return false;
        }
        // The normal duration-1, explosion-free rocket, attached to this player.
        // Item use owns the arm swing; normal item damage handles fan wear/breakage.
        let at = self.player.as_ref().unwrap().position;
        let at = horizon_core::geometry::Vec3::new(at.x, at.y, at.z);
        let item = crate::items::ItemStack::new("minecraft:firework_rocket", 1);
        let mut rocket = self.new_rocket(at, &item, true);
        rocket.owner = Some(crate::mobs::Who::Player);
        rocket.guest = self.lan_acting;
        self.rockets.push(rocket);
        self.add_cooldown(std::sync::Arc::from(FAN), 200);
        if !self.creative {
            let loss = self.wear(stack, 5);
            let max = self.items().get(FAN).map_or(250, |i| i.max_damage);
            let slot = self.held_slot();
            let broken = if let Some(held) = self.held_mut().as_mut() {
                held.damage = held.damage.saturating_add(loss);
                if held.damage >= max {
                    let mut gone = held.clone();
                    gone.count = 1;
                    if held.count > 1 {
                        held.count -= 1;
                        held.damage = 0;
                    } else {
                        *self.held_mut() = None;
                    }
                    Some(gone)
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(gone) = broken {
                self.award_stat("broken", &gone.item, 1);
                let hand = if slot == super::shield::OFFHAND_SLOT {
                    horizon_world::player_inventory::EquipmentSlot::OffHand
                } else {
                    horizon_world::player_inventory::EquipmentSlot::MainHand
                };
                self.player_equipment_broke(hand, &gone);
            }
        }
        if let (Some(player), Some(name)) = (self.lan_acting_uuid, self.lan_acting_name.clone()) {
            self.native_player_elytra_boost(player, &name);
        }
        true
    }

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
        let (size, health, hunger, fairy, racial_glider) =
            self.lunatic_character(id).map_or((1.0, 20.0, 20, false, false), |c| {
                (c.size, c.health, c.hunger, c.has_race("fairy"), c.has_race("crow_tengu") || c.has_race("devil"))
            });
        let potion = self.potions.effects.get("slow_falling").is_some();
        if let Some(p) = self.player.as_mut() {
            p.lunatic_character_body(size, fairy, potion, racial_glider);
            if self.remote.is_some() {
                // Food packets already contain the authoritative amount; do not scale it again.
                p.health.food.max_food=hunger;
                p.health.food.food=p.health.food.food.min(hunger);
            } else { p.health.food.set_capacity(hunger); }
            let has = p.attributes.key("minecraft:max_health").and_then(|key|
                p.attributes.has_modifier(Some(key), "lunatic:character_health").ok()).unwrap_or(false);
            if p.__horizon_mixins_lunatic_fairy().character_health != health || has != (health != 20.0) {
                let _ = p.set_modifier("minecraft:max_health", "lunatic:character_health",
                    f64::from(health) - 20.0, "add_value", health != 20.0);
                p.__horizon_mixins_lunatic_fairy_mut().character_health = health;
            }
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
        self.lunatic_parasol_publish();
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
        let deaths = self.stats.get("custom", "minecraft:deaths");
        let Some(character) = self.lunatic_character(id) else { return; };
        if !self.lunatic_character_confirmed(id) { return; }
        let Some(p) = self.player.as_ref() else {
            return;
        };
        if p.health.dead_or_dying()
            || self.spectator()
            || p.__horizon_mixins_lunatic_fairy().kit_checked.as_ref()
                .is_some_and(|(previous, count)| previous == &character.id && *count == deaths)
        {
            return;
        }
        // Clone only when a kit needs resolving, never on ordinary player ticks.
        let character = Some(character.clone());
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
                .kit_checked = Some((character.as_ref().unwrap().id.clone(), deaths));
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
                        .kit_checked = Some((character.as_ref().unwrap().id.clone(), deaths));
                    return;
                }
            },
            Some(_) => {
                self.player
                    .as_mut()
                    .unwrap()
                    .__horizon_mixins_lunatic_fairy_mut()
                    .kit_checked = Some((character.as_ref().unwrap().id.clone(), deaths));
                return;
            }
        };
        let current = &character.as_ref().unwrap().id;
        // Old saves recorded their first awarded character; adopt it as a safe
        // migration baseline. Empty/unknown legacy histories are not destructive.
        let previous = kits.character.as_ref().or_else(|| kits.awarded.last());
        let switched = previous.is_some_and(|previous| previous != current);
        if switched {
            for slot in self.inventory.iter_mut().chain(self.armor.iter_mut())
                .chain(std::iter::once(&mut self.offhand)) {
                if slot.as_ref().is_some_and(|item| item.enchantments().iter()
                    .any(|(name, level)| name.as_ref() == "lunatic:soulbound" && *level > 0)) {
                    *slot = None;
                }
            }
            kits.pending.retain(|stack| stack.enchantments.get("lunatic:soulbound").copied().unwrap_or(0) == 0);
            self.apply_equipment();
        }
        let identity_changed = kits.character.as_ref() != Some(current);
        kits.character = Some(current.clone());
        let respawned = kits.deaths.is_some_and(|previous| deaths > previous);
        let mut changed = kits.deaths != Some(deaths) || identity_changed;
        kits.deaths = Some(deaths);
        if respawned {
            kits.initialized = false;
            kits.awarded.clear();
            kits.pending.clear();
        }
        let roll_kit = !kits.initialized || switched || respawned;
        if roll_kit {
            kits.initialized = true;
            changed = true;
            // Switching characters may grant a new kit after the old Soulbound
            // stacks are removed. Large retained inventories do not gain another kit.
            kits.pending.clear();
            if self.inventory.iter().flatten().count() < 12
                && let Some(character) = &character
            {
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
                            enchantments: entry.enchantments.clone(),
                        });
                    }
                }
                kits.awarded.push(character.id.clone());
            }
        }
        for stack in &mut kits.pending {
            // Removed items are skipped, including an item removed since the kit was rolled.
            if self.items().get(&stack.item).is_none() {
                stack.count = 0;
                changed = true;
                continue;
            }
            let mut item = crate::items::ItemStack::new(stack.item.as_str(), stack.count);
            for (enchantment, level) in &stack.enchantments {
                item.enchant(enchantment, *level);
            }
            let left = self.native_give_stack(id, item).map_or(0, |item| item.count);
            changed |= left != stack.count;
            stack.count = left;
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
                .kit_checked = Some((character.as_ref().unwrap().id.clone(), deaths));
        }
    }
}
