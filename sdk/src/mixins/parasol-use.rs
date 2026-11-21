// SPDX-License-Identifier: GPL-3.0-or-later
fn use_key(game: &crate::game::Game<'_>, key: InteractionKey, swing: &mut bool, canceled: &mut bool) {
    use crate::game::lunatic_parasol::GameLunaticParasolFloatExt as _;
    if key == InteractionKey::Use && game.on("lunatic") && game.lunatic_parasol_attack_hand(true).is_some()
    {
        // One request is sent by the physical press; held-key repeats do nothing.
        *swing = false;
        *canceled = true;
    }
}
