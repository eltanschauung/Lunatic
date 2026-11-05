// SPDX-License-Identifier: GPL-3.0-or-later
fn enchant(stack: &mut ItemStack, id: &str) -> core::ops::ControlFlow<()> {
    if crate::item_data::lunatic_parasol::is_parasol(stack) && id == "minecraft:knockback" {
        crate::item_data::lunatic_parasol::normalize(stack);
        core::ops::ControlFlow::Break(())
    } else { core::ops::ControlFlow::Continue(()) }
}
