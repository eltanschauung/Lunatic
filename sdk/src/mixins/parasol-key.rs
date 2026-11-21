// SPDX-License-Identifier: GPL-3.0-or-later
fn parasol_press(game: &mut crate::game::Game<'_>, key: &str, action: mod_api::KeyAction) {
    use crate::game::lunatic_parasol::GameLunaticParasolFloatExt as _;
    game.lunatic_parasol_key(key, action == mod_api::KeyAction::Press);
}
