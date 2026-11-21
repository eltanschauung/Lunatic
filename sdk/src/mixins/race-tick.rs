// SPDX-License-Identifier: GPL-3.0-or-later
fn tick(game: &mut crate::game::Game<'_>) {
    use crate::game::lunatic_races::GameLunaticRacesExt as _;
    game.lunatic_race_tick();
}
