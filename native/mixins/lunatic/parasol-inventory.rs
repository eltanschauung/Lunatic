// SPDX-License-Identifier: GPL-3.0-or-later
fn inventory(game: &mut crate::game::Game<'_>, slot: i32) {
    if !game.native_on("lunatic") { return; }
    if let Some(stack) = game.native_carried_stack(slot) {
        horizon_client::item_data::lunatic_parasol::normalize(stack);
    }
}
