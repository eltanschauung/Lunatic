// SPDX-License-Identifier: GPL-3.0-or-later
fn tick(p: &mut crate::particles::Particle) -> std::ops::ControlFlow<bool> {
    if lunatic_particles::tick(p) {
        std::ops::ControlFlow::Break(true)
    } else {
        std::ops::ControlFlow::Continue(())
    }
}
