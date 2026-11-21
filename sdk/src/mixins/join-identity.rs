// SPDX-License-Identifier: GPL-3.0-or-later
use super::super::Game;
impl GameLunaticJoinIdentityExt for Game<'_> {
     fn lunatic_character_confirmed(&self, id: u128) -> bool {
        self.lan_acting.is_none() || self.character_sync.server.selection_confirmed(id)
    }
}

pub trait GameLunaticJoinIdentityExt {
    fn lunatic_character_confirmed(&self, id: u128) -> bool;
}
