// SPDX-License-Identifier: GPL-3.0-or-later
fn disconnect(game: &crate::game::Game<'_>) {
    use crate::game::lunatic_gourd::GameLunaticGourdExt as _;
 game.gourd_stop(); }
