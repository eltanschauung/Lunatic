// Chest motion adapted from Female Gender Mod 5.0.0 by pupnewfster.
// SPDX-License-Identifier: LGPL-3.0-or-later
// Original: com.wildfire.client.physics.BreastPhysics, provided mc26.3 jar.
// Lunatic fixes bounce multiplier and floppiness to 10%, with no user config.
#[derive(Clone, Copy, Debug, Default)]
pub struct Input {
    pub tick: u32,
    pub position: [f64; 3],
    pub yaw: f32,
    pub vertical_velocity: f32,
    pub speed_squared: f32,
    pub walk: f32,
    pub walk_speed: f32,
    pub swing: f32,
    pub crouching: bool,
}
#[derive(Clone, Debug, Default)]
pub struct Physics {
    last: Option<Input>,
    value: [f32; 3],
    previous: [f32; 3],
    velocity: [f32; 3],
    spring: [f32; 3],
    size: f32,
    vertical_sign: f32,
}
#[derive(Debug, Default)]
pub struct Motion(pub std::sync::Mutex<Physics>);
impl Clone for Motion {
    fn clone(&self) -> Self {
        Self(std::sync::Mutex::new(
            self.0
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
        ))
    }
}
pub fn bust(chest: f32, slim: bool) -> f32 {
    if !slim || !chest.is_finite() {
        0.0
    } else {
        (chest.clamp(1.0, 2.0) - 1.0) * 0.8
    }
}
pub fn motion_enabled(chest: f32) -> bool {
    chest >= 1.3 && chest.is_finite()
}
impl Physics {
    pub fn sample(&mut self, input: Input, chest: f32, alpha: f32) -> [f32; 3] {
        if !motion_enabled(chest) {
            *self = Self::default();
            return [0.0; 3];
        }
        let size = bust(chest, true);
        let reset = self.last.is_none_or(|last| {
            input.tick < last.tick
                || input.tick - last.tick > 5
                || input
                    .position
                    .iter()
                    .zip(last.position)
                    .any(|(a, b)| (a - b).abs() > 4.0)
        }) || (self.size - size).abs() > f32::EPSILON;
        if reset {
            *self = Self {
                last: Some(input),
                size,
                vertical_sign: 1.0,
                ..Self::default()
            };
            self.spring[1] = size * 1.25;
            self.value[1] = self.spring[1];
            self.previous = self.value;
        } else if let Some(last) = self.last.filter(|last| last.tick != input.tick) {
            self.previous = self.value;
            let elapsed = input.tick - last.tick;
            let dy = ((input.position[1] - last.position[1]) / f64::from(elapsed)) as f32;
            let yaw = (input.yaw - last.yaw + 180.0).rem_euclid(360.0) - 180.0;
            let intensity = size * 3.0 * 0.3; // round(0.10 * 3 * 100) / 100
            if last.vertical_velocity <= 0.0 && input.vertical_velocity > 0.0
                || last.vertical_velocity < 0.0 && input.vertical_velocity == 0.0
            {
                self.vertical_sign = -self.vertical_sign;
            }
            let rotation = -yaw / 15.0 * intensity;
            let speed = (input.speed_squared / 0.2).powi(3).max(1.0);
            let mut target = [
                -rotation / 10.0,
                dy * intensity
                    + size * 1.25
                    + (input.walk * 0.6662 + std::f32::consts::PI).cos() * 0.25 * input.walk_speed
                        / speed,
                rotation + dy * intensity * self.vertical_sign,
            ];
            if input.crouching != last.crouching {
                target[1] += intensity;
            }
            if input.swing > 0.0 {
                target[1] += 0.25 * intensity * (input.swing * std::f32::consts::TAU).sin();
                target[0] += 0.325 * intensity;
                target[2] -= 0.2 * intensity;
            }
            // Same damped spring integration as the source, at fixed 20 Hz.
            for _ in 0..elapsed {
                let mut target = target;
                if self.spring[1] < -0.5 {
                    target[1] += (self.spring[1] + 1.5).abs() * 0.5;
                }
                if self.spring[1] > 2.5 {
                    target[1] -= (self.spring[1] - 2.65).abs() * 0.5;
                }
                target[1] = target[1].clamp(-1.5, 2.5);
                target[2] = target[2].clamp(-25.0, 25.0);
                let bounce = 0.45 * (1.0 - 0.10) + 0.15;
                let delta = 2.25 - bounce;
                for axis in 0..3 {
                    self.velocity[axis] +=
                        ((target[axis] - self.spring[axis]) * delta - self.velocity[axis]) * bounce;
                    self.spring[axis] +=
                        self.velocity[axis] * 0.10 * if axis == 1 { 1.1625 } else { 1.0 };
                }
                self.value = self.spring;
                self.value[1] = self.value[1].clamp(-0.5, 1.5);
                if self.spring[1] > 1.5 {
                    self.velocity[1] = 0.0;
                }
            }
            self.last = Some(input);
        }
        std::array::from_fn(|i| {
            self.previous[i] + (self.value[i] - self.previous[i]) * alpha.clamp(0.0, 1.0)
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn limits_and_cutoff() {
        assert_eq!(bust(2.0, true), 0.8);
        assert_eq!(bust(1.0, true), 0.0);
        assert_eq!(bust(2.0, false), 0.0);
        assert_eq!(bust(f32::NAN, true), 0.0);
        assert!(!motion_enabled(1.2999));
        assert!(motion_enabled(1.3));
        assert_eq!(
            Physics::default().sample(Input::default(), 1.2, 1.0),
            [0.0; 3]
        );
    }
    #[test]
    fn movement_is_bounded_interpolated_and_frame_independent() {
        let mut a = Physics::default();
        let mut b = Physics::default();
        let mut moved = false;
        for tick in 0..1200 {
            let input = Input {
                tick,
                position: [0.0, (tick as f64 * 0.2).sin(), 0.0],
                yaw: tick as f32 * 3.0,
                vertical_velocity: (tick as f32 * 0.2).cos(),
                walk: tick as f32,
                walk_speed: 0.8,
                crouching: tick % 50 < 25,
                swing: if tick % 10 < 5 { 0.5 } else { 0.0 },
                ..Input::default()
            };
            let result = a.sample(input, 2.0, 1.0);
            for _ in 0..6 {
                b.sample(input, 2.0, 0.3);
            }
            assert_eq!(result, b.sample(input, 2.0, 1.0));
            assert!(result.iter().all(|v| v.is_finite() && v.abs() < 30.0));
            moved |= (result[1] - 1.0).abs() > 0.01;
        }
        assert!(moved);
        assert_eq!(a.sample(Input::default(), 1.0, 1.0), [0.0; 3]);
    }
}
