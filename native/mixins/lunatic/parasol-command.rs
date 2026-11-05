// SPDX-License-Identifier: GPL-3.0-or-later
fn created(mut value: ItemStack) -> ItemStack {
    horizon_client::item_data::lunatic_parasol::normalize(&mut value);
    value
}
