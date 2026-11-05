// SPDX-License-Identifier: GPL-3.0-or-later
fn receive(game: &mut crate::game::Game<'_>, channel: &str, data: &[u8]) {
    game.lunatic_receive_parasol(channel, data);
    game.lunatic_receive_bursts(channel, data);
}
