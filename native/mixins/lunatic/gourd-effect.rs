// SPDX-License-Identifier: GPL-3.0-or-later
fn drink(game: &mut crate::game::Game<'_>, kind: &str) {
    use horizon_core::random::RandomSource;
    if game.native_on("lunatic") && game.remote.is_none() && kind == "lunatic:drink_gourd" {
        // Resolve the authoritative acting player's catalog traits, including LAN guests.
        let oni = game.lunatic_has_race("oni");
        let (effect, seconds) = if oni {
            (
                "lunatic:oni",
                12 + game.world.random.next_int_bound(9).unwrap_or(0),
            )
        } else {
            (
                "lunatic:drunk",
                6 + game.world.random.next_int_bound(5).unwrap_or(0),
            )
        };
        game.add_player_effect(horizon_client::effects::EffectInstance::new(
            effect,
            seconds * 20,
            0,
        ));
    }
}
