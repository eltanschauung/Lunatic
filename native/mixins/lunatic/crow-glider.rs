// SPDX-License-Identifier: GPL-3.0-or-later
fn crow_glider(game: &crate::game::Game<'_>) -> core::ops::ControlFlow<bool> {
    if game.lunatic_crow_tengu() {
        core::ops::ControlFlow::Break(
            game.player
                .as_ref()
                .is_some_and(|p| !p.__horizon_mixins_lunatic_fairy().crow_armor_blocked),
        )
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
