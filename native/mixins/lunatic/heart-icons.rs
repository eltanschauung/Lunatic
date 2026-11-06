// SPDX-License-Identifier: GPL-3.0-or-later
fn heart_icons(atlas: &mut HudAtlas) {
    // Run once before the atlas's first frame, including after a pack reload.
    // Later frames already have their atlas packed and skip all map work.
    if atlas.packed.is_some() || atlas.pixels.is_empty() {
        return;
    }
    // Keep the complete source UVs; the HUD draws each heart at 9x9.
    for name in ["full", "half", "container"] {
        let custom = format!("lunatic:gui/sprites/hud/heart/{name}");
        if let Some(rect) = atlas.sprites.get(&custom).copied() {
            atlas.sprites.insert(format!("gui/sprites/hud/heart/{name}"), rect);
        }
    }
}
