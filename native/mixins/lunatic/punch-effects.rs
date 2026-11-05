// SPDX-License-Identifier: GPL-3.0-or-later
// weapon_hit runs only after an accepted melee hit, including empty hands.
fn landed(game: &mut crate::game::Game<'_>, index: usize) {
    game.lunatic_punch_effects(index);
}
