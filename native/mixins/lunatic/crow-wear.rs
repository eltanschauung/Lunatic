// SPDX-License-Identifier: GPL-3.0-or-later
fn crow_wear(game: &crate::game::Game<'_>) -> core::ops::ControlFlow<()> {
    if game.lunatic_crow_tengu() {
        core::ops::ControlFlow::Break(())
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
