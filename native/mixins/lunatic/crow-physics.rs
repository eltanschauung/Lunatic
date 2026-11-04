// SPDX-License-Identifier: GPL-3.0-or-later
// Calibrated level-flight equilibrium, retaining vanilla pitch steering and collisions.
use horizon_core::geometry::Vec3;
pub const FORWARD: f64 = 24.0 / 20.0;
pub const RATIO: f64 = 7.0;
const SINK: f64 = FORWARD / RATIO;
const VERTICAL_DRAG: f64 = 0.98_f32 as f64;
const VERTICAL_KEEP: f64 = 0.9 * VERTICAL_DRAG;
const DOWN_FORCE: f64 = SINK * (1.0 - VERTICAL_KEEP) / VERTICAL_KEEP;
pub const LIFT: f64 = 1.0 - DOWN_FORCE / 0.08;
pub const HORIZONTAL_DRAG: f64 = FORWARD / (FORWARD + 0.09 * (SINK + DOWN_FORCE));
pub fn lift(mut velocity: Vec3, gravity: f64, look_lift: f64) -> Vec3 {
    velocity.y -= gravity * look_lift * (0.75 - LIFT);
    velocity
}
pub fn drag(velocity: Vec3) -> Vec3 {
    let scale = HORIZONTAL_DRAG / (0.99_f32 as f64);
    Vec3::new(velocity.x * scale, velocity.y, velocity.z * scale)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn level_equilibrium_is_twenty_four_blocks_per_second_at_seven_to_one() {
        let down = 0.08 * (1.0 - LIFT);
        let sink = 0.9 * VERTICAL_DRAG * down / (1.0 - 0.9 * VERTICAL_DRAG);
        let forward = HORIZONTAL_DRAG * 0.09 * (sink + down) / (1.0 - HORIZONTAL_DRAG);
        assert!((forward * 20.0 - 24.0).abs() < 1e-9);
        assert!((forward / sink - 7.0).abs() < 1e-9);
    }
    #[test]
    fn adjustments_preserve_heading_and_do_not_replace_vertical_drag() {
        let v = Vec3::new(0.6, -0.2, 0.8);
        let changed = drag(v);
        assert!((changed.x / changed.z - v.x / v.z).abs() < 1e-12);
        assert_eq!(changed.y, v.y);
        assert_eq!(lift(v, 0.08, 0.0), v);
    }
}
