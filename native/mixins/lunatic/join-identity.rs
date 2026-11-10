// SPDX-License-Identifier: GPL-3.0-or-later
use super::super::Game;
impl Game<'_> {
    pub(crate) fn lunatic_character_confirmed(&self, id: u128) -> bool {
        self.lan_acting.is_none() || self.character_sync.server.selection_confirmed(id)
    }
}
