// SPDX-License-Identifier: GPL-3.0-or-later
fn flight(
    projectile: &mut Projectile,
    world: &dyn ProjectileWorld,
) -> std::ops::ControlFlow<(bool, Hit)> {
    match lunatic_motion::tick(projectile, world) {
        Some(result) => std::ops::ControlFlow::Break(result),
        None => std::ops::ControlFlow::Continue(()),
    }
}
