// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
pub(super) const REQUEST: &str = "lunatic:parasol_fire";
pub(super) const STATE: &str = "lunatic:parasol_state";
const ITEM: &str = "lunatic:yuuka_parasol";

impl Game<'_> {
    pub(super) fn lunatic_parasol_hand(&self) -> Option<u8> {
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

    pub(super) fn lunatic_parasol_key(&mut self, key: &str, pressed: bool) {
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

    pub(super) fn lunatic_parasol_input(&mut self, alternate: bool) -> bool {
        if !self.native_on("lunatic") || !self.native_playing() || self.spectator() {
            return false;
        }
        let Some(hand) = self.lunatic_parasol_hand() else {
            return false;
        };
        if self.remote.is_some() {
            let _ = self.native_net_send_to_server(REQUEST, &[hand, u8::from(alternate)]);
        } else {
            self.lunatic_yuuka_fire(hand, alternate);
        }
        true
    }

    pub(super) fn lunatic_parasol_tick(&mut self) {
        let holding = self
            .held()
            .into_iter()
            .chain(self.offhand.as_ref())
            .any(|stack| stack.item.as_ref() == ITEM);
        let Some(player) = self.player.as_mut() else {
            return;
        };
        let was_active = player.__horizon_mixins_lunatic_fairy().parasol;
        let active = holding
            && !player.ground.on_ground
            && !player.flying
            && !player.spectator
            && !player.is_passenger
            && !player.sleeping
            && !player.fall_flying
            && !player.fluids.was_touching_water
            && !player.is_in_lava()
            && !player.health.dead_or_dying()
            && (was_active || (player.velocity.y < 0.0 && player.fall_distance > 3.0));
        if active != was_active {
            player.lunatic_parasol_set(active);
            if active {
                let at = player.position;
                self.lunatic_sound("item.yuuka_parasol.open", at, 0.5);
            }
        }
    }

    pub(super) fn lunatic_receive_parasol(&mut self, channel: &str, data: &[u8]) {
        if channel != STATE
            || data.len() != 1
            || data[0] > 1
            || !self.native_on("lunatic")
            || self.remote.is_none()
        {
            return;
        }
        if let Some(player) = self.player.as_mut() {
            player.lunatic_parasol_set(data[0] != 0);
        }
    }
}
