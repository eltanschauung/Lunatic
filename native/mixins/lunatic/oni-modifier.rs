// SPDX-License-Identifier: GPL-3.0-or-later
fn modifier(
    effect: &str,
) -> core::ops::ControlFlow<Option<(&'static str, &'static str, f64, &'static str)>> {
    if effect == "lunatic:oni" {
        core::ops::ControlFlow::Break(Some((
            "minecraft:attack_damage",
            "lunatic:effect.oni",
            6.0,
            "add_value",
        )))
    } else {
        core::ops::ControlFlow::Continue(())
    }
}
