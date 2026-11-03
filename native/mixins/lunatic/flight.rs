// SPDX-License-Identifier: GPL-3.0-or-later
fn flight(
    projectile: &mut Projectile,
    world: &dyn ProjectileWorld,
) -> std::ops::ControlFlow<(bool, Hit)> {
    if let Some(result)=crate::projectiles::lampad_motion::tick(projectile,world) {return std::ops::ControlFlow::Break(result);}
    match lunatic_motion::tick(projectile, world) {
        Some(result) => std::ops::ControlFlow::Break(result),
        None => std::ops::ControlFlow::Continue(()),
    }
}
