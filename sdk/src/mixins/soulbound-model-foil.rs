// SPDX-License-Identifier: GPL-3.0-or-later
fn model_foil(components: &ComponentsView<'_>) -> core::ops::ControlFlow<Option<bool>> {
    let bound = components.get("minecraft:enchantments").is_some_and(|enchants| {
        enchants.get("levels").unwrap_or(enchants).get("lunatic:soulbound").and_then(|value| value.as_i64()).is_some_and(|n| n > 0)
    });
    if bound { core::ops::ControlFlow::Break(Some(false)) } else { core::ops::ControlFlow::Continue(()) }
}
