// SPDX-License-Identifier: GPL-3.0-or-later
fn before(game: &mut crate::game::Game<'_>) {
    use crate::game::lunatic_fairy_motion::PlayerMotionLunaticFairyMotionExt as _;
    let ending = game.on("lunatic")
        && game.remote.is_none()
        && game
            .potions
            .effects
            .get("lunatic:oni")
            .is_some_and(|effect| effect.duration == 1);
    if let Some(player) = game.player.as_mut() {
        let ending = ending && !player.health.dead_or_dying();
        player.lunatic_edit_fairy(|state| state.oni_ending = ending);
    }
}
