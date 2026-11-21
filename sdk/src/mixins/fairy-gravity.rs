// SPDX-License-Identifier: GPL-3.0-or-later
fn gravity(world: &mut crate::motion::MotionWorld<'_, '_, '_>) {
    use crate::motion::native_motion::lunatic_fairy::PlayerMotionLunaticFairyMotionExt as _;
    let crow_tengu = world.player.lunatic_fairy().racial_glider;
    world.player.glider |= crow_tengu;
    world.player.status.slow_falling |= world.player.lunatic_fairy().slowfall;
}
