// SPDX-License-Identifier: GPL-3.0-or-later
fn receive(game: &mut crate::game::Game<'_>) {
    use crate::game::lunatic_effects::GameLunaticEffectsExt as _;
    use crate::game::lunatic_parasol::GameLunaticParasolFloatExt as _;
    use crate::game::lunatic_gourd::GameLunaticGourdExt as _;
    for (channel, data) in ::mod_lunatic::take_replies() {
        game.gourd_receive(channel, &data);
        game.lunatic_receive_parasol(channel, &data);
        game.lunatic_receive_bursts(channel, &data);
    }
}
