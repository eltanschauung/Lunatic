// SPDX-License-Identifier: GPL-3.0-or-later
fn hook(
    p: &mut horizon_client::projectiles::Projectile,
    world: &dyn horizon_client::projectiles::ProjectileWorld,
) -> std::ops::ControlFlow<()> {
    if !p.kind.id().starts_with("lunatic:lampad_star_") {
        return std::ops::ControlFlow::Continue(());
    }
    // The server owns the accelerating curve and sends its velocity. Replaying
    // the absolute curve from a client-local origin/age fights position updates.
    // Extrapolate instead; previous/current retain normal per-frame interpolation.
    p.set_old_pos_and_rot();
    p.age = p.age.saturating_add(1);
    let next = p.position.add_vector(p.velocity);
    if let Some((at, _, _)) = world.clip(p.position, next) {
        p.position = at;
        p.velocity = horizon_core::geometry::Vec3::ZERO;
    } else {
        p.position = next;
    }
    // Hits, lifetime, damage and removal remain authoritative on the server.
    std::ops::ControlFlow::Break(())
}
