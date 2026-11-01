// SPDX-License-Identifier: GPL-3.0-or-later
// Server-owned burst events, grouped by nearby recipient and reconstructed on clients.
use super::Game;
use horizon_client::projectiles::lunatic_motion as motion;
use std::cell::RefCell;
pub(super) const CHANNEL: &str = "lunatic:orb_bursts";
const RECORD: usize = 41;
#[derive(Clone, Debug, PartialEq)]
struct Burst {
    variant: u8,
    bounce: bool,
    normal: [i8; 3],
    at: [f64; 3],
    dir: [f32; 3],
}
thread_local! { static QUEUE: RefCell<Vec<(&'static str,Burst)>> = const { RefCell::new(Vec::new()) }; }
fn encode(dimension: &str, bursts: &[Burst]) -> Vec<u8> {
    let mut data = vec![1, dimension.len() as u8];
    data.extend_from_slice(dimension.as_bytes());
    data.push(bursts.len() as u8);
    for b in bursts {
        data.extend_from_slice(&[
            b.variant,
            u8::from(b.bounce),
            b.normal[0] as u8,
            b.normal[1] as u8,
            b.normal[2] as u8,
        ]);
        for v in b.at {
            data.extend_from_slice(&v.to_be_bytes());
        }
        for v in b.dir {
            data.extend_from_slice(&v.to_be_bytes());
        }
    }
    data
}
fn decode(data: &[u8], dimension: &str) -> Option<Vec<Burst>> {
    if data.first() != Some(&1) {
        return None;
    }
    let len = usize::from(*data.get(1)?);
    if data.get(2..2 + len)? != dimension.as_bytes() {
        return None;
    }
    let count = usize::from(*data.get(2 + len)?);
    if count > 128 || data.len() != 3 + len + count * RECORD {
        return None;
    }
    let mut bursts = Vec::with_capacity(count);
    for r in data[3 + len..].chunks_exact(RECORD) {
        let normal = [r[2] as i8, r[3] as i8, r[4] as i8];
        let at = std::array::from_fn(|i| {
            f64::from_be_bytes(r[5 + i * 8..13 + i * 8].try_into().expect("fixed record"))
        });
        let dir = std::array::from_fn(|i| {
            f32::from_be_bytes(r[29 + i * 4..33 + i * 4].try_into().expect("fixed record"))
        });
        if r[0] > 2
            || r[1] > 1
            || normal.iter().map(|v| i16::from(*v).abs()).sum::<i16>() != 1
            || at.iter().any(|v| !v.is_finite() || v.abs() > 30_000_000.0)
            || dir.iter().any(|v| !v.is_finite())
        {
            return None;
        }
        bursts.push(Burst {
            variant: r[0],
            bounce: r[1] == 1,
            normal,
            at,
            dir,
        });
    }
    Some(bursts)
}
impl Game<'_> {
    pub(super) fn lunatic_queue_burst(&self, event: &motion::Event) {
        if event.sound == motion::Sound::CloseDetonation {
            return;
        } // Impact follows it: emit once.
        let Some(variant) = motion::variant(&event.projectile) else {
            return;
        };
        let p = event.position;
        let v = event.projectile.velocity;
        let burst = Burst {
            variant: variant as u8,
            bounce: event.sound == motion::Sound::Bounce,
            normal: event.normal,
            at: [p.x, p.y, p.z],
            dir: [v.x as f32, v.y as f32, v.z as f32],
        };
        QUEUE.with_borrow_mut(|q| {
            if q.len() < 512 {
                q.push((self.dim.id(), burst));
            }
        });
    }
    pub(super) fn lunatic_flush_bursts(&mut self) {
        let queued = QUEUE.with_borrow_mut(std::mem::take);
        for p in self.native_server_players() {
            let bursts: Vec<_> = queued
                .iter()
                .filter(|(dim, b)| {
                    *dim == p.dimension
                        && (0..3)
                            .map(|i| (p.position[i] - b.at[i]).powi(2))
                            .sum::<f64>()
                            <= 32.0 * 32.0
                })
                .map(|(_, b)| b.clone())
                .collect();
            for batch in bursts.chunks(128) {
                let _ =
                    self.native_net_send_to_player(p.uuid, CHANNEL, &encode(&p.dimension, batch));
            }
        }
    }
    pub(super) fn lunatic_receive_bursts(&mut self, channel: &str, data: &[u8]) {
        if channel != CHANNEL || !self.native_on("lunatic") || self.remote.is_none() {
            return;
        }
        let Some(bursts) = decode(data, self.dim.id()) else {
            return;
        };
        let Some(player) = &self.player else {
            return;
        };
        for b in bursts {
            if (player.position.x - b.at[0]).powi(2)
                + (player.position.y - b.at[1]).powi(2)
                + (player.position.z - b.at[2]).powi(2)
                > 32.0 * 32.0
            {
                continue;
            }
            self.particles.lunatic_orb_burst(
                b.at,
                b.dir.map(f64::from),
                b.normal,
                b.variant,
                b.bounce,
            );
        }
    }
    pub(super) fn lunatic_clear_bursts(&self) {
        QUEUE.with_borrow_mut(Vec::clear);
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compact_batch_roundtrip_rejects_bad_data_and_wrong_dimension() {
        let b = Burst {
            variant: 2,
            bounce: false,
            normal: [0, 1, 0],
            at: [1.5, 70.0, -2.5],
            dir: [0.0, 0.0, 1.25],
        };
        let data = encode("minecraft:overworld", &[b.clone()]);
        assert_eq!(decode(&data, "minecraft:overworld"), Some(vec![b]));
        assert!(decode(&data, "minecraft:the_nether").is_none());
        for len in 0..data.len() {
            assert!(decode(&data[..len], "minecraft:overworld").is_none());
        }
        let mut bad = data.clone();
        let last = bad.len() - 4;
        bad[last..].copy_from_slice(&f32::NAN.to_be_bytes());
        assert!(decode(&bad, "minecraft:overworld").is_none());
    }
}
