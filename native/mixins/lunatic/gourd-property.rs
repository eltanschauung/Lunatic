// SPDX-License-Identifier: GPL-3.0-or-later
// Mixin-only mods have no mod_api::Hooks entry: answer before Env's empty-registry shortcut.
fn model_property(env: &Env<'_, '_>, property: &str) -> std::ops::ControlFlow<bool> {
    if property == "lunatic:gourd_deployed" {
        std::ops::ControlFlow::Break(env.game.gourd_deployed())
    } else {
        std::ops::ControlFlow::Continue(())
    }
}
