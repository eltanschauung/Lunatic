// SPDX-License-Identifier: GPL-3.0-or-later
fn knockback(
    game: &mut crate::game::Game<'_>,
    target_kind: &str,
    scale: f32,
) -> core::ops::ControlFlow<(f32, bool)> {
    let minimum = if game.potions.effects.get("lunatic:oni").is_some() {
        2.0
    } else if game.lunatic_has_race("yuuka") {
        1.0
    } else { 0.0 };
    if !game.native_on("lunatic") || minimum == 0.0 {
        return core::ops::ControlFlow::Continue(());
    }
    // Keep the normal data-driven enchantment evaluation, sprint bonus and
    // attack-strength rules; Oni supplies Knockback II; Yuuka supplies Knockback I.
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
        enchanted.max(minimum),
        scale,
        sprinting,
    ))
}
