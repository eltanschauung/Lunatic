// SPDX-License-Identifier: GPL-3.0-or-later
fn gravity(world: &mut crate::motion::MotionWorld<'_, '_, '_>) {
    world.player.status.slow_falling |= world.player.__horizon_mixins_lunatic_fairy().slowfall;
}
