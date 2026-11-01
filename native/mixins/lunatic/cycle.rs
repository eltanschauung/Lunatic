// SPDX-License-Identifier: GPL-3.0-or-later
fn cycle(game: &mut crate::game::Game<'_>, stack: &ItemStack, claimed: &mut bool) {
    game.lunatic_cycle(stack, claimed);
}
