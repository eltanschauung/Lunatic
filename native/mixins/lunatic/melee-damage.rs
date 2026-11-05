// SPDX-License-Identifier: GPL-3.0-or-later
fn damage(value: f32, game: &crate::game::Game<'_>) -> f32 {
    value + game.lunatic_melee_bonus()
}
