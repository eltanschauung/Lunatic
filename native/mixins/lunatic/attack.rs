// SPDX-License-Identifier: GPL-3.0-or-later
fn attack(game: &mut crate::game::Game<'_>) -> std::ops::ControlFlow<bool> {
    if game.lunatic_input() {
        std::ops::ControlFlow::Break(true)
    } else {
        std::ops::ControlFlow::Continue(())
    }
}
