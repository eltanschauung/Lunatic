// SPDX-License-Identifier: GPL-3.0-or-later
fn guest(game: &mut crate::game::Game<'_>, host: &mut crate::game::lan::LanHost, conn: u32) {
    game.lunatic_guest_races(host, conn);
}
