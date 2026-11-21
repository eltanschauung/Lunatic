// SPDX-License-Identifier: GPL-3.0-or-later
fn body(game: &mut crate::game::Game<'_>) {
    use crate::game::lunatic_races::GameLunaticRacesExt as _;
    use crate::game::lunatic_fairy_motion::PlayerMotionLunaticFairyMotionExt as _;
    game.lunatic_character_body();
}
