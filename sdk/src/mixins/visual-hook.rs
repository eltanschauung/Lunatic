// SPDX-License-Identifier: GPL-3.0-or-later
fn visuals(
    game: &crate::game::Game<'_>,
    lights: &mut Vec<mod_api::ColoredLight>,
) {
    use crate::game::lunatic_visuals::GameLunaticVisualsExt as _;
    game.lunatic_visuals(lights);
}
