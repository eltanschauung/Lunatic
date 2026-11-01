// SPDX-License-Identifier: GPL-3.0-or-later
fn tick(p: &mut crate::particles::Particle) -> std::ops::ControlFlow<()> {
    if crate::particles::lunatic_particles::tick(p) {
        std::ops::ControlFlow::Break(())
    } else {
        std::ops::ControlFlow::Continue(())
    }
}
