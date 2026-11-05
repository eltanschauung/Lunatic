// SPDX-License-Identifier: GPL-3.0-or-later
fn use_key(game: &crate::game::Game<'_>, button: i32, swing: &mut bool, canceled: &mut bool) {
    if button == 1 && game.native_on("lunatic") && game.lunatic_parasol_hand().is_some() {
        // One request is sent by the physical press; held-key repeats do nothing.
        *swing = false;
        *canceled = true;
    }
}
