// SPDX-License-Identifier: GPL-3.0-or-later
fn gravity(world: &mut crate::motion::MotionWorld<'_, '_, '_>) {
    let crow_tengu = world.player.__horizon_mixins_lunatic_fairy().racial_glider;
    world.player.glider |= crow_tengu;
    world.player.status.slow_falling |= world.player.__horizon_mixins_lunatic_fairy().slowfall;
}
