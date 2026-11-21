// SPDX-License-Identifier: GPL-3.0-or-later
use crate::game::lunatic_yuuka::GameLunaticYuukaExt as _;
use crate::game::lunatic_weapons::GameLunaticWeaponsExt as _;
use crate::game::lunatic_fairy_motion::PlayerMotionLunaticFairyMotionExt as _;
use super::{Game, lunatic_weapons::YUUKA_PARASOL};
pub(super) const REQUEST: &str = "lunatic:parasol_fire";
pub(super) const STATE: &str = "lunatic:parasol_state";
pub(super) const FALL_DISTANCE: f64 = 2.0;
const ITEM: &str = YUUKA_PARASOL.item;

impl GameLunaticParasolFloatExt for Game<'_> {
     fn lunatic_parasol_hand(&self) -> Option<u8> {
        if self.held().is_some_and(|s| s.item.as_ref() == ITEM) {
            Some(0)
        } else if self.held().is_none()
            && self
                .offhand
                .as_ref()
                .is_some_and(|s| s.item.as_ref() == ITEM)
        {
            Some(1)
        } else {
            None
        }
    }

     fn lunatic_parasol_attack_hand(&self, alternate: bool) -> Option<u8> {
        self.lunatic_parasol_hand()
            .filter(|_| self.lunatic_action_allowed(YUUKA_PARASOL.attack(alternate)))
    }

     fn lunatic_parasol_key(&mut self, key: &str, pressed: bool) {
        if pressed
            && self
                .settings
                .keys
                .mappings_for(key)
                .any(|mapping| mapping == "key.use")
        {
            self.lunatic_parasol_input(true);
        }
    }

     fn lunatic_parasol_input(&mut self, alternate: bool) -> bool {
        if !self.on("lunatic") || !self.playing() || self.spectator() {
            return false;
        }
        let Some(hand) = self.lunatic_parasol_attack_hand(alternate) else {
            return false;
        };
        if self.remote.is_some() {
            let _ = super::lunatic_orbs::send_to_server(self, REQUEST, &[hand, u8::from(alternate)]);
        } else {
            self.lunatic_yuuka_fire(hand, alternate);
        }
        true
    }

     fn lunatic_parasol_tick(&mut self) {
        let hand = if self.held().is_some_and(|stack| stack.item.as_ref() == ITEM) {
            Some(0)
        } else if self
            .offhand
            .as_ref()
            .is_some_and(|stack| stack.item.as_ref() == ITEM)
        {
            Some(1)
        } else {
            None
        };
        let Some(player) = self.player.as_mut() else {
            return;
        };
        let was_active = player.lunatic_fairy().parasol;
        let active = hand.is_some()
            && !player.ground.on_ground
            && !player.flying
            && !player.spectator
            && !player.is_passenger
            && !player.sleeping
            && !player.fall_flying
            && !player.fluids.was_touching_water
            && !player.is_in_lava()
            && !player.health.dead_or_dying()
            && (was_active || (player.velocity.y < 0.0 && player.fall_distance > FALL_DISTANCE));
        if active != was_active {
            player.lunatic_parasol_set(active);
            if active && let Some(hand) = hand {
                let at = player.position;
                self.lunatic_wear_weapon(&YUUKA_PARASOL, YUUKA_PARASOL.passive, hand == 1);
                self.lunatic_action_sound(YUUKA_PARASOL.passive, at);
            }
        }
    }

     fn lunatic_receive_parasol(&mut self, channel: &str, data: &[u8]) {
        if channel != STATE
            || data.len() != 1
            || data[0] > 1
            || !self.on("lunatic")
            || self.remote.is_none()
        {
            return;
        }
        if let Some(player) = self.player.as_mut() {
            player.lunatic_parasol_set(data[0] != 0);
        }
    }
}

pub trait GameLunaticParasolFloatExt {
    fn lunatic_parasol_hand(&self) -> Option<u8>;
    fn lunatic_parasol_attack_hand(&self, alternate: bool) -> Option<u8>;
    fn lunatic_parasol_key(&mut self, key: &str, pressed: bool);
    fn lunatic_parasol_input(&mut self, alternate: bool) -> bool;
    fn lunatic_parasol_tick(&mut self);
    fn lunatic_receive_parasol(&mut self, channel: &str, data: &[u8]);
}
