// SPDX-License-Identifier: GPL-3.0-or-later
fn heart_icons(atlas: &mut HudAtlas) {
    // Run once before the atlas's first frame, including after a pack reload.
    if atlas.packed.is_some() || atlas.pixels.is_empty() {
        return;
    }
    // Discover supplied variants once. New textures then need only a content rebuild.
    // Preserve complete source UVs; the existing HUD draws every heart at 9x9.
    let aliases: Vec<_> = atlas.sprites.iter().filter_map(|(key, rect)| {
        let name = key.strip_prefix("lunatic:gui/sprites/hud/heart/")?;
        let vanilla = format!("gui/sprites/hud/heart/{name}");
        atlas.sprites.contains_key(&vanilla).then_some((vanilla, *rect))
    }).collect();
    for (name, rect) in aliases {
        atlas.sprites.insert(name, rect);
    }
}
