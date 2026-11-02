// SPDX-License-Identifier: GPL-3.0-or-later
fn held_name(game: &crate::game::Game<'_>) -> std::ops::ControlFlow<Option<HeldName>> {
    match game.lunatic_held_name() {
        Some(name) => std::ops::ControlFlow::Break(Some(name)),
        None => std::ops::ControlFlow::Continue(()),
    }
}
