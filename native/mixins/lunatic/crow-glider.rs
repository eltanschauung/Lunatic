// SPDX-License-Identifier: GPL-3.0-or-later
fn crow_glider(game: &crate::game::Game<'_>) -> core::ops::ControlFlow<bool> {
    if game.lunatic_crow_tengu() {
        core::ops::ControlFlow::Break(true)
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
