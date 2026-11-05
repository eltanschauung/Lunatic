// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
pub(super) const REQUEST: &str = "lunatic:parasol_toggle";
pub(super) const STATE: &str = "lunatic:parasol_state";
const ITEM: &str = "lunatic:yuuka_parasol";
impl Game<'_> {
    pub(super) fn lunatic_parasol_hand(&self) -> Option<u8> {
        if self.held().is_some_and(|s| s.item.as_ref() == ITEM) { Some(0) }
        else if self.held().is_none() && self.offhand.as_ref().is_some_and(|s| s.item.as_ref() == ITEM) { Some(1) }
        else { None }
    }
    pub(super) fn lunatic_parasol_key(&mut self, key: &str, pressed: bool) {
        if !pressed || !self.native_on("lunatic") || !self.native_playing()
            || !self.settings.keys.mappings_for(key).any(|mapping| mapping == "key.use") {
            return;
        }
        let Some(hand) = self.lunatic_parasol_hand() else { return; };
        if self.remote.is_some() {
            let _ = self.native_net_send_to_server(REQUEST, &[hand]);
        } else { self.lunatic_parasol_toggle(hand); }
    }
    pub(super) fn lunatic_parasol_toggle(&mut self, hand: u8) {
        let slot = if hand == 0 { self.selected } else { super::shield::OFFHAND_SLOT };
        if self.remote.is_some() || self.spectator() || !self.native_on("lunatic")
            || !self.stack_in(slot).is_some_and(|s| s.item.as_ref() == ITEM) { return; }
        let Some(player) = self.player.as_mut().filter(|p| !p.health.dead_or_dying()) else { return; };
        let active = !player.__horizon_mixins_lunatic_fairy().parasol;
        player.lunatic_parasol_set(active);
        if !active {
            player.__horizon_mixins_lunatic_fairy_mut().sound = Some("entity.fairy.cancel");
        } else {
            let at = player.position;
            // Minecraft's six wing clips, selected through the standard sound event.
            self.audio.play("minecraft:entity.ender_dragon.flap", Some([at.x, at.y, at.z]), 0.5, 1.0);
        }
    }
    pub(super) fn lunatic_receive_parasol(&mut self, channel: &str, data: &[u8]) {
        if channel != STATE || data.len() != 1 || data[0] > 1
            || !self.native_on("lunatic") || self.remote.is_none() { return; }
        if let Some(player) = self.player.as_mut() { player.lunatic_parasol_set(data[0] != 0); }
    }
}
