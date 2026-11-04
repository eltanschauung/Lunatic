// SPDX-License-Identifier: GPL-3.0-or-later
fn after(value: (), game: &mut crate::game::Game<'_>) {
    let () = value;
    let Some(player) = game.player.as_mut() else {
        return;
    };
    let ended = std::mem::take(&mut player.__horizon_mixins_lunatic_fairy_mut().oni_ending);
    // A hidden effect taking over is not an expiry. Death and early removal
    // never reach this branch; only the authoritative last tick does.
    if !ended || player.health.dead_or_dying() || game.potions.effects.get("lunatic:oni").is_some()
    {
        return;
    }
    let loss = 2 + game.world.random.next_int_bound(4).unwrap_or(0);
    let food = &mut player.health.food;
    food.food = (food.food - loss).max(0);
    food.saturation = food.saturation.min(food.food as f32);
    let at = [player.position.x, player.position.y, player.position.z];
    let seconds = 6 + game.world.random.next_int_bound(5).unwrap_or(0);
    game.add_player_effect(horizon_client::effects::EffectInstance::new(
        "lunatic:drunk",
        seconds * 20,
        0,
    ));
    // Exactly FoodProperties' eating-completion burp gain and pitch range.
    let pitch = game.world.random.next_float() * (1.0f32 - 0.9f32) + 0.9;
    game.audio
        .play("minecraft:entity.player.burp", Some(at), 0.5, pitch);
}
