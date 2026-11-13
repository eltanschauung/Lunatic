// SPDX-License-Identifier: GPL-3.0-or-later
fn foil(stack: &ItemStack) -> core::ops::ControlFlow<bool> {
    if stack.enchantments().iter().any(|(id, level)| id.as_ref() == "lunatic:soulbound" && *level > 0) {
        core::ops::ControlFlow::Break(false)
    } else { core::ops::ControlFlow::Continue(()) }
}
