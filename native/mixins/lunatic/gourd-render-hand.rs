fn hand(game: &crate::game::Game<'_>, mob: &horizon_client::mobs::Mob, camera: glam::DVec3, alpha: f64) {
    game.gourd_render_hand(mob,camera,alpha);
}
