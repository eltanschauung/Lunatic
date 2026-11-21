// SPDX-License-Identifier: GPL-3.0-or-later
fn crow_drag(value: Vec3, world: &crate::motion::MotionWorld<'_, '_, '_>) -> Vec3 {
    use crate::motion::native_motion::lunatic_fairy::PlayerMotionLunaticFairyMotionExt as _;
    if world.player.lunatic_fairy().racial_glider {
        crate::motion::native_motion::lunatic_crow_physics::drag(value)
    } else {
        value
    }
}
