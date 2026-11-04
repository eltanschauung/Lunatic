// SPDX-License-Identifier: GPL-3.0-or-later
fn crow_lift(value: Vec3, world: &mut MotionWorld<'_, '_, '_>) -> Vec3 {
    if !world.player.__horizon_mixins_lunatic_fairy().crow_tengu { return value; }
    let lean = world.player.pitch * trig::DEG_TO_RAD;
    let lift = horizon_core::jmath::cos(f64::from(lean)).powi(2);
    // Recover the already-resolved gravity from vanilla's initial vertical step.
    // This preserves gravity attributes and Slow Falling without a second query.
    let gravity = (world.player.velocity.y - value.y) / (1.0 - 0.75 * lift);
    crate::motion::lunatic_crow_physics::lift(value, gravity, lift)
}
