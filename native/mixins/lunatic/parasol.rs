// SPDX-License-Identifier: GPL-3.0-or-later
use crate::items::ItemStack;
pub fn is_parasol(stack: &ItemStack) -> bool { stack.item.as_ref() == "lunatic:yuuka_parasol" }
pub fn normalize(stack: &mut ItemStack) {
    if !is_parasol(stack) || stack.enchantment_level("minecraft:knockback") == 3 { return; }
    let levels = &mut stack.data_mut().enchantments;
    levels.retain(|(id, _)| id.as_ref() != "minecraft:knockback");
    levels.push((std::sync::Arc::from("minecraft:knockback"), 3));
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn creation_and_enchantment_mutation_keep_knockback_three() {
        let mut s = ItemStack::new("lunatic:yuuka_parasol", 1);
        normalize(&mut s);
        assert_eq!(s.enchantment_level("minecraft:knockback"), 3);
        s.set_crafting_enchantments(vec![]);
        assert_eq!(s.enchantment_level("minecraft:knockback"), 3);
        s.enchant("minecraft:knockback", 4);
        assert_eq!(s.enchantment_level("minecraft:knockback"), 3);
        s.enchant("minecraft:sharpness", 2);
        assert_eq!(s.enchantment_level("minecraft:knockback"), 3);
        assert_eq!(s.enchantment_level("minecraft:sharpness"), 2);
        assert_eq!(ItemStack::new("minecraft:iron_sword", 1).enchantment_level("minecraft:knockback"), 0);
    }
}
