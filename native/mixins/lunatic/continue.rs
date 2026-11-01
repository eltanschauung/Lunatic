// SPDX-License-Identifier: GPL-3.0-or-later
fn held_attack(game: &mut crate::game::Game<'_>) -> std::ops::ControlFlow<()> {
    if game.lunatic_continue() {
        std::ops::ControlFlow::Break(())
    } else {
        std::ops::ControlFlow::Continue(())
    }
}
