// SPDX-License-Identifier: GPL-3.0-or-later
fn parasol_press(game: &mut crate::game::Game<'_>, key: &str, pressed: bool) {
    game.lunatic_parasol_key(key, pressed);
}
