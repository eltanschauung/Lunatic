// SPDX-License-Identifier: GPL-3.0-or-later
// Per-tick spring/damper integration. Collision and replication use Horizon's projectile APIs.
pub const LENGTH: f64 = 16.0;
pub const LIMIT: f64 = 17.4;
pub const NOZZLE: f64 = 10.3 / 16.0;
pub const RADIUS: f64 = 0.45;
pub const HALF_SEGMENT: f64 = 0.30;
pub const DAMAGE: f32 = 16.0;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Thrown,
    Recoiling,
    Falling,
    Grounded,
    Retracting,
}
#[derive(Clone, Debug)]
pub struct Body {
    pub at: [f64; 3],
    pub velocity: [f64; 3],
    pub phase: Phase,
    pub recoil_ticks: u32,
}
pub fn add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] + b[i])
}
pub fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}
pub fn mul(a: [f64; 3], n: f64) -> [f64; 3] {
    a.map(|v| v * n)
}
pub fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    (0..3).map(|i| a[i] * b[i]).sum()
}
pub fn length(a: [f64; 3]) -> f64 {
    dot(a, a).sqrt()
}
pub fn forward(yaw: f32, pitch: f32) -> [f64; 3] {
    let y = f64::from(yaw).to_radians();
    let p = f64::from(pitch).to_radians();
    [-y.sin() * p.cos(), -p.sin(), y.cos() * p.cos()]
}
pub fn nozzle(at: [f64; 3], axis: [f64; 3]) -> [f64; 3] {
    sub(at, mul(axis, NOZZLE))
}
pub fn right_hand(eye: [f64; 3], dir: [f64; 3]) -> [f64; 3] {
    [eye[0] - dir[2] * 0.38, eye[1] - 0.3, eye[2] + dir[0] * 0.38]
}
pub fn third_person_hand(eye: [f64;3], dir: [f64;3], scale: f64) -> [f64;3] {
    let horizontal = (dir[0]*dir[0]+dir[2]*dir[2]).sqrt().max(1e-6);
    let x=dir[0]/horizontal; let z=dir[2]/horizontal;
    [eye[0]+(-z*0.3125+x*0.35)*scale, eye[1]-0.77*scale, eye[2]+(x*0.3125+z*0.35)*scale]
}
impl Body {
    pub fn new(at: [f64; 3], velocity: [f64; 3]) -> Self {
        Self {
            at,
            velocity,
            phase: Phase::Thrown,
            recoil_ticks: 0,
        }
    }
    pub fn retract(&mut self) {
        self.phase = Phase::Retracting;
    }
    /// Returns true only when a manually/exceptionally retracted body reaches the hand.
    pub fn step(&mut self, anchor: [f64; 3]) -> bool {
        if self.phase == Phase::Retracting {
            let delta = sub(anchor, self.at);
            let d = length(delta);
            if d <= 0.4 {
                return true;
            }
            self.velocity = mul(delta, 1.8_f64.min(d) / d);
            self.at = add(self.at, self.velocity);
            return false;
        }
        if self.phase == Phase::Grounded {
            if length(sub(self.at, anchor)) <= LIMIT {
                return false;
            }
            self.phase = Phase::Falling;
        }
        for _ in 0..8 {
            let delta = sub(self.at, anchor);
            let d = length(delta);
            let n = if d > 1e-8 {
                mul(delta, 1.0 / d)
            } else {
                [0.0; 3]
            };
            if self.phase == Phase::Thrown && d >= LENGTH {
                self.phase = Phase::Recoiling;
            }
            if self.phase == Phase::Recoiling {
                let radial = dot(self.velocity, n);
                // A tension-only spring, followed by one damped rebound, never an ongoing homing force.
                let tension = (2.2 * (d - LENGTH).max(0.0) + 0.55 * radial.max(0.0)).max(0.0);
                self.velocity = sub(self.velocity, mul(n, tension / 8.0));
                if self.recoil_ticks > 0 && radial < -0.7 {
                    self.phase = Phase::Falling;
                }
            }
            let gravity = match self.phase {
                Phase::Thrown => 0.003,
                Phase::Recoiling => 0.02,
                _ => 0.05,
            };
            self.velocity[1] -= gravity / 8.0;
            self.at = add(self.at, mul(self.velocity, 1.0 / 8.0));
            let delta = sub(self.at, anchor);
            let d = length(delta);
            if d > LIMIT {
                let n = mul(delta, 1.0 / d);
                self.at = add(anchor, mul(n, LIMIT));
                let v = dot(self.velocity, n);
                if v > 0.0 {
                    self.velocity = sub(self.velocity, mul(n, v * 1.18));
                }
            }
        }
        if self.phase == Phase::Recoiling {
            self.recoil_ticks += 1;
            if self.recoil_ticks >= 20 {
                self.phase = Phase::Falling;
            }
        }
        self.velocity[0] *= 0.995;
        self.velocity[2] *= 0.995;
        false
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn crosshair_flight_is_straight_before_spring_and_stretch_is_visible() {
        let mut b = Body::new([0.0, 0.0, 0.9], [0.0, 0.0, 1.65]);
        for _ in 0..8 {
            assert!(!b.step([0.0; 3]));
        }
        assert!(b.at[2] > 13.0 && b.at[1].abs() < 0.15);
        let mut peak = 0.0_f64;
        let mut rebound = 0.0_f64;
        for _ in 0..18 {
            b.step([0.0; 3]);
            peak = peak.max(length(b.at));
            rebound = rebound.min(b.velocity[2]);
        }
        assert!(
            peak > LENGTH + 0.5 && peak <= LIMIT + 1e-8,
            "stretch {peak}"
        );
        assert!(rebound < -0.7, "rebound {rebound}");
    }
    #[test]
    fn cork_and_nozzle_trail_the_impact_end_and_hand_is_on_right() {
        for (yaw, pitch) in [(0.0, 0.0), (90.0, 20.0), (-120.0, -40.0)] {
            let axis = forward(yaw, pitch);
            let at = mul(axis, 10.0);
            let attachment = nozzle(at, axis);
            assert!(dot(sub(attachment, at), axis) < -0.6);
            assert!((length(sub(attachment, at)) - NOZZLE).abs() < 1e-8);
        }
        let right = right_hand([0.0; 3], forward(0.0, 0.0));
        assert!(right[0] < 0.0 && right[1] < 0.0);
    }
    #[test]
    fn spring_rebounds_then_falls_and_never_returns() {
        let mut b = Body::new([0.0; 3], [0.0, 0.1, 1.5]);
        let mut recoil = false;
        let mut rebound = false;
        for _ in 0..100 {
            assert!(!b.step([0.0; 3]));
            assert!(length(b.at) <= LIMIT + 1e-8);
            recoil |= b.phase == Phase::Recoiling;
            rebound |= b.velocity[2] < 0.0;
        }
        assert!(recoil && rebound);
        assert_eq!(b.phase, Phase::Falling);
    }
    #[test]
    fn resting_requires_second_click() {
        let mut b = Body::new([0.0, -1.0, 5.0], [0.0; 3]);
        b.phase = Phase::Grounded;
        for _ in 0..1000 {
            assert!(!b.step([0.0; 3]));
            assert_eq!(b.at, [0.0, -1.0, 5.0]);
        }
        b.retract();
        assert!((0..10).any(|_| b.step([0.0; 3])));
    }
    #[test]
    fn any_deployed_phase_can_retract() {
        for phase in [
            Phase::Thrown,
            Phase::Recoiling,
            Phase::Falling,
            Phase::Grounded,
        ] {
            let mut b = Body::new([0.0, 0.0, 7.0], [0.0; 3]);
            b.phase = phase;
            b.retract();
            assert!((0..10).any(|_| b.step([0.0; 3])));
        }
    }
}
