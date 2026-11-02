// SPDX-License-Identifier: GPL-3.0-or-later
// Luanti's purification_projectile.pointlight, tinted to each orb's inventory color.
// Read client entities each frame: no packets, saved light writes or retained effects.
use super::{Game, native_hooks::NativeColoredLight};
use horizon_client::projectiles::{Projectile, lunatic_motion};
use horizon_core::geometry::Vec3;
const ORB_INTENSITY: f32 = 0.25;
const BURST_INTENSITY: f32 = ORB_INTENSITY * 0.5;
// Half HSV saturation: blend toward the brightest channel, keeping value fixed.
fn light_rgb(variant: usize) -> u32 {
    let [_, r, g, b] = [0xdc143cu32, 0x800080, 0x00ff00][variant].to_be_bytes();
    let white = u32::from(r.max(g).max(b));
    let half = |v: u8| (u32::from(v) + white + 1) / 2;
    (half(r) << 16) | (half(g) << 8) | half(b)
}
fn source(at: Vec3, variant: usize, intensity: f32) -> NativeColoredLight {
    NativeColoredLight {
        at: [at.x, at.y, at.z],
        rgb: light_rgb(variant),
        radius: 6.0,
        intensity,
    }
}
fn held(at: Vec3, main: Option<&str>, off: Option<&str>) -> Vec<NativeColoredLight> {
    let mut seen = None;
    [main, off]
        .into_iter()
        .flatten()
        .filter_map(|item| {
            let variant = lunatic_motion::ITEMS.iter().position(|id| *id == item)?;
            if seen == Some(variant) {
                return None;
            }
            seen = Some(variant);
            Some(source(at, variant, ORB_INTENSITY))
        })
        .collect()
}
fn nearest(
    sources: impl IntoIterator<Item = NativeColoredLight>,
    origin: Vec3,
    loaded: impl Fn((i32, i32)) -> bool,
) -> Vec<NativeColoredLight> {
    let mut selected: Vec<(f64, NativeColoredLight)> = Vec::with_capacity(8);
    for light in sources {
        let at = light.at;
        let distance =
            (at[0] - origin.x).powi(2) + (at[1] - origin.y).powi(2) + (at[2] - origin.z).powi(2);
        if !distance.is_finite()
            || distance > 32.0 * 32.0
            || !loaded((
                (at[0].floor() as i32).div_euclid(16),
                (at[2].floor() as i32).div_euclid(16),
            ))
        {
            continue;
        }
        let index = selected.partition_point(|(d, _)| *d <= distance);
        if index < 8 {
            selected.insert(index, (distance, light));
            selected.truncate(8);
        }
    }
    selected.into_iter().map(|(_, l)| l).collect()
}

fn lights(
    projectiles: &[Projectile],
    origin: Vec3,
    alpha: f64,
    loaded: impl Fn((i32, i32)) -> bool,
) -> Vec<NativeColoredLight> {
    nearest(
        projectiles.iter().filter_map(|p| {
            let variant = lunatic_motion::variant(p)?;
            Some(source(
                p.previous.lerp(p.position, alpha),
                variant,
                ORB_INTENSITY,
            ))
        }),
        origin,
        loaded,
    )
}

impl Game<'_> {
    pub(super) fn lunatic_visuals(&self, out: &mut Vec<NativeColoredLight>) {
        if !self.native_on("lunatic") {
            return;
        }
        let Some(player) = &self.player else {
            return;
        };
        let alpha = self.clock.fraction();
        let mut sources = lights(
            &self.creatures.projectiles,
            player.position,
            alpha,
            |chunk| self.world.chunks.contains_key(&chunk),
        );
        if !player.spectator {
            let at = self
                .previous_position
                .lerp(player.position, self.clock.camera_fraction());
            sources.extend(held(
                Vec3::new(at.x, at.y + 1.2, at.z),
                self.held().map(|s| s.item.as_ref()),
                self.offhand.as_ref().map(|s| s.item.as_ref()),
            ));
        }
        for m in self
            .mobs
            .iter()
            .filter(|m| !m.removed && !m.extra.spectator)
        {
            let at = m.previous.lerp(m.position, alpha);
            sources.extend(held(
                Vec3::new(at.x, at.y + f64::from(m.height()) * 0.6, at.z),
                m.extra.main_hand.as_deref(),
                m.extra.off_hand.as_deref(),
            ));
        }
        for (at, variant) in self.particles.lunatic_orb_origins() {
            sources.push(source(
                Vec3::new(at[0], at[1], at[2]),
                usize::from(variant),
                BURST_INTENSITY,
            ));
        }
        out.extend(nearest(sources, player.position, |chunk| {
            self.world.chunks.contains_key(&chunk)
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use horizon_client::projectiles::{ProjectileKind, ThrownType};
    static TYPES: [ThrownType; 3] = [
        ThrownType {
            id: lunatic_motion::ITEMS[0],
            item: lunatic_motion::ITEMS[0],
            damage: 1.0,
        },
        ThrownType {
            id: lunatic_motion::ITEMS[1],
            item: lunatic_motion::ITEMS[1],
            damage: 0.5,
        },
        ThrownType {
            id: lunatic_motion::ITEMS[2],
            item: lunatic_motion::ITEMS[2],
            damage: 2.0,
        },
    ];
    fn orb(kind: usize, x: f64) -> Projectile {
        Projectile::new(
            ProjectileKind::Thrown(&TYPES[kind]),
            Vec3::new(x, 0.0, 0.0),
            None,
        )
    }
    #[test]
    fn luanti_pointlight_for_all_three_variants() {
        let orbs = [orb(0, 1.0), orb(1, 2.0), orb(2, 3.0)];
        let out = lights(&orbs, Vec3::ZERO, 1.0, |_| true);
        assert_eq!(out.len(), 3);
        for (l, rgb) in out.iter().zip([0xdc788c, 0x804080, 0x80ff80]) {
            assert_eq!((l.rgb, l.radius, l.intensity), (rgb, 6.0, 0.25));
        }
    }
    #[test]
    fn nearest_loaded_sources_follow_render_interpolation() {
        let mut orbs: Vec<_> = (1..=12).rev().map(|x| orb(0, f64::from(x))).collect();
        for p in &mut orbs {
            p.previous.x = p.position.x - 2.0;
        }
        // Unloaded sources must not consume the eight-request budget.
        orbs.push(orb(1, -1.0));
        orbs.push(orb(2, 100.0));
        let out = lights(&orbs, Vec3::ZERO, 0.5, |(x, _)| x >= 0);
        assert_eq!(
            out.iter().map(|l| l.at[0]).collect::<Vec<_>>(),
            (0..8).map(f64::from).collect::<Vec<_>>()
        );
    }
    #[test]
    fn held_orbs_match_travelling_and_bursts_are_half_strength() {
        let hand = held(
            Vec3::ZERO,
            Some(lunatic_motion::ITEMS[1]),
            Some(lunatic_motion::ITEMS[1]),
        );
        assert_eq!(hand.len(), 1);
        assert_eq!(
            (hand[0].rgb, hand[0].radius, hand[0].intensity),
            (0x804080, 6.0, 0.25)
        );
        assert_eq!(
            held(
                Vec3::ZERO,
                Some(lunatic_motion::ITEMS[0]),
                Some(lunatic_motion::ITEMS[2])
            )
            .len(),
            2
        );
        assert!(held(Vec3::ZERO, Some("minecraft:stick"), None).is_empty());
        let burst = source(Vec3::ZERO, 1, BURST_INTENSITY);
        assert_eq!(burst.intensity, hand[0].intensity * 0.5);
    }
    #[test]
    fn no_stale_lights_or_vanilla_projectile_lights() {
        let snowball = Projectile::new(ProjectileKind::Snowball, Vec3::ZERO, None);
        assert!(lights(&[snowball], Vec3::ZERO, 1.0, |_| true).is_empty());
        assert_eq!(lights(&[orb(0, 0.0)], Vec3::ZERO, 1.0, |_| true).len(), 1);
        assert!(lights(&[], Vec3::ZERO, 1.0, |_| true).is_empty());
    }
}
