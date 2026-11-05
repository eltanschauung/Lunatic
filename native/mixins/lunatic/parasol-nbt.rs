// SPDX-License-Identifier: GPL-3.0-or-later
// All creative stacks and saved/loot NBT enter here; normalize after decoding their patch.
fn decoded(tag: &Tag) -> core::ops::ControlFlow<Option<crate::items::ItemStack>> {
    if text(tag.get("id")).as_deref() != Some("lunatic:yuuka_parasol") {
        return core::ops::ControlFlow::Continue(());
    }
    let count = number(tag.get("count")).or_else(|| number(tag.get("Count")))
        .filter(|count| (1..=99).contains(count)).unwrap_or(1) as u32;
    let (damage, extra) = crate::item_data::decode_components(tag.get("components"));
    let mut stack = crate::items::ItemStack { item: Arc::from("lunatic:yuuka_parasol"), count, damage, extra };
    crate::item_data::lunatic_parasol::normalize(&mut stack);
    core::ops::ControlFlow::Break(Some(stack))
}
