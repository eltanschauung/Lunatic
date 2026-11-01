// SPDX-License-Identifier: GPL-3.0-or-later
fn network(game: &mut crate::game::Game<'_>, player: u128, channel: &str, data: &[u8]) {
    game.lunatic_message(player, channel, data);
}
