// SPDX-License-Identifier: GPL-3.0-or-later
fn grindstone(input: Option<&ItemStack>, additional: Option<&ItemStack>) -> core::ops::ControlFlow<Option<ItemStack>> {
    // An intrinsic enchantment cannot be stripped or repeatedly redeemed for XP.
    if [input, additional].into_iter().flatten().any(horizon_client::item_data::lunatic_parasol::is_parasol) {
        core::ops::ControlFlow::Break(None)
    } else { core::ops::ControlFlow::Continue(()) }
}
