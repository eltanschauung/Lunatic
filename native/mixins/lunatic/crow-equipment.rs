// SPDX-License-Identifier: GPL-3.0-or-later
fn equipment(game: &mut crate::game::Game<'_>) {
    if let Some(player) = game.player.as_mut() {
        player.lunatic_crow_glide();
    }
}
