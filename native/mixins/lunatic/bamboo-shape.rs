// SPDX-License-Identifier: GPL-3.0-or-later
// Bamboo's cached shape is intentionally absent: its stalk is offset by position.
// Renaming its states must preserve the template's dynamic collision AND support.
fn bamboo_shape(state: &crate::chunk::BlockState)
    -> std::ops::ControlFlow<std::io::Result<Option<crate::dynamic_support::DynamicSupport>>> {
    if state.name != "lunatic:bushy_bamboo" {
        return std::ops::ControlFlow::Continue(());
    }
    let mut template = state.clone();
    template.name = "minecraft:bamboo".into();
    std::ops::ControlFlow::Break(crate::dynamic_support::DynamicSupport::from_state(&template))
}
