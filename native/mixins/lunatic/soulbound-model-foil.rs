// SPDX-License-Identifier: GPL-3.0-or-later
fn model_foil(resolver: &Resolve<'_>) -> core::ops::ControlFlow<bool> {
    let components = resolver.components;
    let bound = components.get("minecraft:enchantments").is_some_and(|enchants| {
        enchants.get("levels").unwrap_or(enchants).get("lunatic:soulbound").and_then(serde_json::Value::as_i64).is_some_and(|n| n > 0)
    });
    if bound { core::ops::ControlFlow::Break(false) } else { core::ops::ControlFlow::Continue(()) }
}
