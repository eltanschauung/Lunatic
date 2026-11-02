// SPDX-License-Identifier: GPL-3.0-or-later
fn step(world: &mut crate::motion::MotionWorld<'_, '_, '_>, jump: bool) {
    let grounded = world.player.ground.on_ground;
    world.player.lunatic_fairy_step(jump, grounded);
}
