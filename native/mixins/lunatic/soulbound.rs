// SPDX-License-Identifier: GPL-3.0-or-later
// Soulbound is a distinct enchantment, currently sharing Vanishing's behavior.
fn level(stack: &ItemStack, id: &str) -> core::ops::ControlFlow<u32> {
    if id != "minecraft:vanishing_curse" {
        return core::ops::ControlFlow::Continue(());
    }
    core::ops::ControlFlow::Break(stack.enchantments().iter()
        .filter(|(id, _)| matches!(id.as_ref(), "minecraft:vanishing_curse" | "lunatic:soulbound"))
        .map(|(_, level)| *level).max().unwrap_or(0))
}
