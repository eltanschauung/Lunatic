// SPDX-License-Identifier: GPL-3.0-or-later
fn donation_entity(game: &crate::game::Game<'_>, block: &str, entity: &mut Option<(String, Compound)>) {
    if game.native_on("lunatic") && block == "lunatic:donation_box" {
        *entity = Some(("lunatic:donation_box".into(), Compound::new()));
    }
}
