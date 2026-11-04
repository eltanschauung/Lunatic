// SPDX-License-Identifier: GPL-3.0-or-later
fn knockback(
    game: &mut crate::game::Game<'_>,
    target_kind: &str,
    scale: f32,
) -> core::ops::ControlFlow<(f32, bool)> {
    if !game.native_on("lunatic") || game.potions.effects.get("lunatic:oni").is_none() {
        return core::ops::ControlFlow::Continue(());
    }
    // Keep the normal data-driven enchantment evaluation, sprint bonus and
    // attack-strength rules; Oni supplies a minimum of Knockback II.
    let enchanted = match game.held().cloned() {
        Some(held) => {
            let ctx = horizon_client::enchant::EffectContext {
                this: Some(target_kind),
                attacker: Some("minecraft:player"),
                direct_attacker: Some("minecraft:player"),
                is_direct: true,
                items: Some(world_items!(game)),
                ..Default::default()
            };
            world_stations!(game).enchantments.modify_value(
                Some(&held),
                "minecraft:knockback",
                &ctx,
                &mut game.world.random,
                0.0,
            )
        }
        None => 0.0,
    };
    let sprinting = game.player.as_ref().is_some_and(|p| p.sprinting);
    core::ops::ControlFlow::Break(horizon_client::attack_strength::extra_knockback(
        enchanted.max(2.0),
        scale,
        sprinting,
    ))
}
