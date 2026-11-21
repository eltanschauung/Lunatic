// SPDX-License-Identifier: GPL-3.0-or-later
fn guest(game: &mut crate::game::Game<'_>, conn: u32) {
    use crate::game::lunatic_races::GameLunaticRacesExt as _;
    game.lunatic_guest_races(conn);
}
