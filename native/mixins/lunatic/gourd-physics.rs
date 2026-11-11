// SPDX-License-Identifier: GPL-3.0-or-later
// Per-tick spring/damper integration. Collision and replication use Horizon's projectile APIs.
pub const LENGTH: f64 = 8.0;
pub const LIMIT: f64 = 8.4;
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
            self.velocity = mul(delta, 1.3_f64.min(d) / d);
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
                let tension = (6.0 * (d - LENGTH).max(0.0) + 3.6 * radial.max(0.0)).max(0.0);
                self.velocity = sub(self.velocity, mul(n, tension / 8.0));
                if self.recoil_ticks > 0 && radial < -0.025 {
                    self.phase = Phase::Falling;
                }
            }
            self.velocity[1] -= 0.05 / 8.0;
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
        self.velocity[0] *= 0.99;
        self.velocity[2] *= 0.99;
        false
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spring_rebounds_then_falls_and_never_returns() {
        let mut b = Body::new([0.0; 3], [0.0, 0.1, 1.5]);
        let mut recoil = false;
        let mut rebound = false;
        for _ in 0..80 {
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
