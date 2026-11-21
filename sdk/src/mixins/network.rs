// SPDX-License-Identifier: GPL-3.0-or-later
fn network(game: &mut crate::game::Game<'_>) {
    use crate::game::lunatic_orbs::GameLunaticRuntimeExt as _;
    for (player, channel, data) in ::mod_lunatic::take_requests() {
        if game.player(player.uuid()) == Some(player) {
            game.lunatic_message(player.uuid(), channel, &data);
        }
    }
}
