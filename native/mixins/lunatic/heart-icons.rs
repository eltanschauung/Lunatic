// SPDX-License-Identifier: GPL-3.0-or-later
fn heart_icons(mut value: Result<HudAtlas>) -> Result<HudAtlas> {
    if let Ok(atlas) = &mut value {
        // Keep the complete source UVs; the HUD draws each heart at 9x9.
        for name in ["full", "half"] {
            let custom = format!("lunatic:gui/sprites/hud/heart/{name}");
            if let Some(rect) = atlas.sprites.get(&custom).copied() {
                atlas.sprites.insert(format!("gui/sprites/hud/heart/{name}"), rect);
            }
        }
    }
    value
}
