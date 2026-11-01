// SPDX-License-Identifier: GPL-3.0-or-later
fn tooltip(game: &crate::game::Game<'_>, stack: &ItemStack, lines: &mut Vec<(Text, u32)>) {
    game.lunatic_tooltip(stack, lines);
}
