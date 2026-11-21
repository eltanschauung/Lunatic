// SPDX-License-Identifier: GPL-3.0-or-later
pub fn opacity(distance: f64, exempt: bool, revealed: bool) -> f32 {
    if revealed { 1.0 } else if exempt { 0.75 } else { (0.65 - 0.60 * (distance / 32.0).clamp(0.0, 1.0)) as f32 }
}
#[cfg(test)] mod tests {
 use super::*;
 #[test] fn distance_and_exemptions() {
  assert!((opacity(0.,false,false)-0.65).abs()<1e-6);
  assert!((opacity(16.,false,false)-0.35).abs()<1e-6);
  assert!((opacity(32.,false,false)-0.05).abs()<1e-6);
  assert_eq!(opacity(100.,true,false),0.75);assert_eq!(opacity(100.,false,true),1.);
 }
}
