// SPDX-License-Identifier: GPL-3.0-or-later
fn crow_wear(game: &crate::game::Game<'_>) -> core::ops::ControlFlow<()> {
    use crate::game::lunatic_races::GameLunaticRacesExt as _;
    if game.lunatic_racial_glider() {
        core::ops::ControlFlow::Break(())
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
