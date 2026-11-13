// SPDX-License-Identifier: GPL-3.0-or-later
fn finish(game: &mut crate::game::Game<'_>, stack: &ItemStack, result: &mut Option<ItemStack>, hand: horizon_client::first_person::Hand) {
    if game.native_on("lunatic") && stack.item.as_ref() == "lunatic:ibuki_gourd" {
        // Keep the original gourd and all its components after normal consumption.
        // The standard consumable already applied effects, cooldown and use events.
        let slot = match hand {
            horizon_client::first_person::Hand::Main => horizon_world::player_inventory::EquipmentSlot::MainHand,
            horizon_client::first_person::Hand::Off => horizon_world::player_inventory::EquipmentSlot::OffHand,
        };
        *result = game.gourd_wear(stack, 1, slot);
    }
}
