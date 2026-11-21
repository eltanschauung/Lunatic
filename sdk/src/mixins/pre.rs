// SPDX-License-Identifier: GPL-3.0-or-later
fn pre(game: &mut crate::game::Game<'_>) {
    use crate::game::lunatic_orbs::GameLunaticRuntimeExt as _;
    game.lunatic_pre();
}
