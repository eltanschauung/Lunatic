// SPDX-License-Identifier: GPL-3.0-or-later
fn light(
    p: &crate::particles::Particle,
    sky: u32,
    block: u32,
) -> std::ops::ControlFlow<(u32, u32)> {
    match crate::particles::lunatic_particles::light(p, sky, block) {
        Some(light) => std::ops::ControlFlow::Break(light),
        None => std::ops::ControlFlow::Continue(()),
    }
}
