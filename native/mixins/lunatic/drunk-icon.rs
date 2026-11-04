// SPDX-License-Identifier: GPL-3.0-or-later
fn drunk_icon(atlas: &HudAtlas, name: &str) -> core::ops::ControlFlow<Option<Rect>> {
    if matches!(name, "lunatic:mob_effect/drunk" | "mob_effect/drunk") {
        if let Some(sprite) = atlas.sprites.get("item/sweet_berries").copied() {
            return core::ops::ControlFlow::Break(Some(sprite));
        }
    }
    core::ops::ControlFlow::Continue(())
}
