// SPDX-License-Identifier: GPL-3.0-or-later
fn added(game: &mut crate::game::Game<'_>, effect: &str, entity: Option<i32>) {
    if !game.native_on("lunatic") || game.remote.is_some() || effect != "lunatic:oni" {
        return;
    }
    let at = match entity {
        None => game.player.as_ref().map(|p| p.position),
        Some(id) => game.mobs.iter().find(|m| m.id as i32 == id).map(|m| m.position),
    };
    if let Some(at) = at {
        // Vanilla's ambient event selects uniformly from the four dragon growls.
        game.audio.play("minecraft:entity.ender_dragon.ambient", Some([at.x, at.y, at.z]), 0.5, 1.0);
    }
}
