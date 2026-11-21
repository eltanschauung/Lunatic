// SPDX-License-Identifier: GPL-3.0-or-later
fn inventory(game: &mut crate::game::Game<'_>, slot: i32) {
    if !game.on("lunatic") { return; }
    if let Some(stack) = game.native_carried_slot(slot).and_then(Option::as_mut) {
        crate::game::lunatic_parasol_item::normalize(stack);
    }
}
