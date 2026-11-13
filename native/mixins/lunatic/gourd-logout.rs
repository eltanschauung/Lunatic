// SPDX-License-Identifier: GPL-3.0-or-later
fn logout(game: &mut crate::game::Game<'_>, player: u128) { game.gourd_player_left(player); }
