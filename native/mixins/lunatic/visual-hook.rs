// SPDX-License-Identifier: GPL-3.0-or-later
fn visuals(
    game: &crate::game::Game<'_>,
    lights: &mut Vec<crate::game::native_hooks::NativeColoredLight>,
) {
    game.lunatic_visuals(lights);
}
