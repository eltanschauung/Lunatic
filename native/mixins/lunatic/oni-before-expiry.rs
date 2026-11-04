// SPDX-License-Identifier: GPL-3.0-or-later
fn before(game: &mut crate::game::Game<'_>) {
    let ending = game.native_on("lunatic")
        && game.remote.is_none()
        && game
            .potions
            .effects
            .get("lunatic:oni")
            .is_some_and(|effect| effect.duration == 1);
    if let Some(player) = game.player.as_mut() {
        let ending = ending && !player.health.dead_or_dying();
        player.__horizon_mixins_lunatic_fairy_mut().oni_ending = ending;
    }
}
