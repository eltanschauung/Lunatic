// SPDX-License-Identifier: GPL-3.0-or-later
fn donation_container(game: &crate::game::Game<'_>, block: &str, container: &mut Option<ModContainer>) {
    if game.native_on("lunatic") && block == "lunatic:donation_box" {
        let slots: Vec<usize> = (0..45).collect();
        *container = Some(ModContainer {
            slots: 45,
            faces: std::array::from_fn(|_| slots.clone()),
            inputs: slots.clone(),
            outputs: slots,
            menu: Some("minecraft:generic_9x5".into()),
        });
    }
}
