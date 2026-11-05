// SPDX-License-Identifier: GPL-3.0-or-later
fn fov(
    game: &crate::game::Game<'_>,
    player: &horizon_client::camera::FovPlayer,
    base: f32,
    modifier: &mut f32,
) {
    if !game.native_on("lunatic") || game.potions.effects.get("lunatic:oni").is_none() {
        return;
    }
    // Speed I adds 20% movement speed. Virtual +10% gives exactly half its
    // FOV delta through vanilla's linear speed term, without changing motion.
    let mut visual = *player;
    visual.movement_speed *= 1.1;
    *modifier += horizon_client::camera::field_of_view_modifier(
        &visual, game.camera_type.first_person(), game.fov_effect_scale(),
    ) - base;
}
