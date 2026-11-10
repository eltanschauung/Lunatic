// SPDX-License-Identifier: GPL-3.0-or-later
fn crow_drag(value: Vec3, world: &mut MotionWorld<'_, '_, '_>) -> Vec3 {
    if world.player.__horizon_mixins_lunatic_fairy().racial_glider {
        let value = crate::motion::lunatic_crow_physics::drag(value);
        world.player.velocity = value;
        value
    } else {
        value
    }
}
