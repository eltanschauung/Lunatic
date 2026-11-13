// SPDX-License-Identifier: GPL-3.0-or-later
fn fade_hand(mut geometry: (Vec<crate::mesher::Vertex>, Vec<(u32, i32)>), game: &crate::game::Game<'_>) -> (Vec<crate::mesher::Vertex>, Vec<(u32, i32)>) {
    if let Some(mob) = game.player_model.as_ref() {
        crate::game::player_presentation::fade_vertices(&mut geometry.0, game.koishi_opacity(mob, glam::DVec3::ZERO));
    }
    geometry
}
