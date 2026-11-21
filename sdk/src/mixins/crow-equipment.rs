// SPDX-License-Identifier: GPL-3.0-or-later
fn equipment(game: &mut crate::game::Game<'_>) {
    use crate::game::lunatic_fairy_motion::PlayerMotionLunaticFairyMotionExt as _;
    if let Some(player) = game.player.as_mut() {
        player.lunatic_racial_glide();
    }
}
