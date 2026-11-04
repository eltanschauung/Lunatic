// SPDX-License-Identifier: GPL-3.0-or-later
fn finish(game: &crate::game::Game<'_>, stack: &ItemStack, result: &mut Option<ItemStack>) {
    if game.native_on("lunatic") && stack.item.as_ref() == "lunatic:ibuki_gourd" {
        // Keep the original gourd and all its components after normal consumption.
        // The standard consumable already applied effects, cooldown and use events.
        *result = Some(stack.clone());
    }
}
