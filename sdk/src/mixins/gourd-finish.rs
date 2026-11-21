// SPDX-License-Identifier: GPL-3.0-or-later
fn finish(game: &mut crate::game::Game<'_>, stack: &ItemStack, result: &mut Option<ItemStack>, hand: horizon_client::first_person::Hand) {
    use crate::game::lunatic_weapons::GameLunaticWeaponsExt as _;
    if game.on("lunatic") && stack.item.as_ref() == "lunatic:ibuki_gourd" {
        // Keep the original gourd and all its components after normal consumption.
        // The standard consumable already applied effects, cooldown and use events.
        let slot = match hand {
            horizon_client::first_person::Hand::Main => horizon_core::equipment_slot::EquipmentSlot::MainHand,
            horizon_client::first_person::Hand::Off => horizon_core::equipment_slot::EquipmentSlot::OffHand,
        };
        *result = game.lunatic_worn_stack(stack, 1, slot);
    }
}
