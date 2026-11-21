// SPDX-License-Identifier: GPL-3.0-or-later
fn crow_lift(gravity: f64, world: &crate::motion::MotionWorld<'_, '_, '_>) -> f64 {
    use crate::motion::native_motion::lunatic_fairy::PlayerMotionLunaticFairyMotionExt as _;
    if !world.player.lunatic_fairy().racial_glider {
        return gravity;
    }
    let lean = world.player.pitch * horizon_core::geometry::trig::DEG_TO_RAD;
    let lift = horizon_core::jmath::cos(f64::from(lean)).powi(2);
    gravity * (1.0 - crate::motion::native_motion::lunatic_crow_physics::LIFT * lift) / (1.0 - 0.75 * lift)
}
