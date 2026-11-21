// SPDX-License-Identifier: GPL-3.0-or-later
fn created(mut value: ItemStack) -> ItemStack {
    crate::game::lunatic_parasol_item::normalize(&mut value);
    value
}
