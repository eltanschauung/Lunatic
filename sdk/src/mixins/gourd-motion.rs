// SPDX-License-Identifier: GPL-3.0-or-later
use horizon_client_lib::projectiles::{Hit, Projectile, ProjectileWorld};
use ::mod_lunatic::gourd_physics as f;
use horizon_core::geometry::Vec3;
pub const ID: &str = "lunatic:gourd_flail";
pub fn ours(p: &Projectile) -> bool {
    p.kind.id() == ID
}
pub fn arr(v: Vec3) -> [f64; 3] {
    [v.x, v.y, v.z]
}
pub fn vec(a: [f64; 3]) -> Vec3 {
    Vec3::new(a[0], a[1], a[2])
}
pub use ::mod_lunatic::projectile_state::Cue;
use ::mod_lunatic::projectile_state::{GOURD, GourdFlight as Flight, GourdState, with_state};
pub type Event = ::mod_lunatic::projectile_state::GourdEvent<Projectile, Hit>;
pub type CueEvent = ::mod_lunatic::projectile_state::GourdCue<Vec3>;
type State = GourdState<Projectile, Hit, Vec3>;
pub fn begin(level: u64, live: &[u32]) {
    with_state::<State, _>(&GOURD, |s| {
        s.level = level;
        s.flights
            .retain(|(w, id), _| *w != level || live.contains(id));
    });
}
pub fn prepare(id: u32, owner: u128, anchor: [f64; 3], p: &Projectile, retract: bool) {
    with_state::<State, _>(&GOURD, |s| {
        let q = s.flights.entry((s.level, id)).or_insert_with(|| Flight {
            body: f::Body::new(arr(p.position), arr(p.velocity)),
            anchor,
            owner,
            hit: false,
            landed: false,
        });
        q.anchor = anchor;
        if retract {
            q.body.retract();
        }
    });
}
pub fn view(id: u32) -> Option<(u128, [f64; 3], f::Phase)> {
    with_state::<State, _>(&GOURD, |s| {
        s.flights
            .get(&(s.level, id))
            .map(|q| (q.owner, q.anchor, q.body.phase))
    })
}
pub fn cues() -> Vec<CueEvent> {
    with_state::<State, _>(&GOURD, |s| std::mem::take(&mut s.cues))
}
pub fn finish() -> Vec<Event> {
    with_state::<State, _>(&GOURD, |s| std::mem::take(&mut s.events))
}
pub fn reset() {
    with_state::<State, _>(&GOURD, |s| *s = State::default());
}
pub fn tick(p: &mut Projectile, w: &dyn ProjectileWorld) -> Option<(bool, Hit)> {
    if !ours(p) {
        return None;
    }
    Some(with_state::<State, _>(&GOURD, |s| {
        let key = (s.level, p.id);
        let Some(q) = s.flights.get_mut(&key) else {
            return (false, Hit::None);
        };
        p.set_old_pos_and_rot();
        p.age = p.age.saturating_add(1);
        p.left_owner = true;
        p.needs_sync = true;
        if !q
            .body
            .at
            .iter()
            .chain(q.anchor.iter())
            .all(|x| x.is_finite())
        {
            s.flights.remove(&key);
            return (false, Hit::None);
        }
        // Probe the entire footprint: a ledge may support the rim while the center is in air.
        if q.body.phase == f::Phase::Grounded {
            let supported = [-0.4,0.0,0.4].into_iter().any(|x| [-0.4,0.0,0.4].into_iter().any(|z| {
                let at=p.position.add_vector(Vec3::new(x,0.0,z));
                w.clip(at,at.add_vector(Vec3::new(0.0,-0.85,0.0))).is_some_and(|(_,_,face)| face[1]>0)
            }));
            if !supported { q.body.phase=f::Phase::Falling; }
        }
        let old = p.position;
        let phase = q.body.phase;
        if q.body.step(q.anchor) {
            s.flights.remove(&key);
            return (false, Hit::None);
        }
        if phase == f::Phase::Thrown && q.body.phase == f::Phase::Recoiling {
            s.cues.push(CueEvent {
                at: vec(q.body.at),
                kind: Cue::Elastic,
            });
        }
        let next = vec(q.body.at);
        let axis = vec(f::forward(p.yaw, p.pitch));
        if q.body.phase != f::Phase::Retracting {
            // Sweep the gourd's radius against actual block collision shapes.
            let mut end = next;
            let mut block = None;
            let mut best = old.distance_squared(next);
            let radial = [
                Vec3::ZERO,
                Vec3::new(f::RADIUS, 0.0, 0.0),
                Vec3::new(-f::RADIUS, 0.0, 0.0),
                Vec3::new(0.0, -f::RADIUS, 0.0),
                Vec3::new(0.0, f::RADIUS, 0.0),
                Vec3::new(0.0, 0.0, f::RADIUS),
                Vec3::new(0.0, 0.0, -f::RADIUS),
            ];
            for offset in [-f::HALF_SEGMENT, 0.0, f::HALF_SEGMENT]
                .into_iter()
                .flat_map(|n| {
                    let along = Vec3::new(axis.x * n, axis.y * n, axis.z * n);
                    radial.into_iter().map(move |r| r.add_vector(along))
                })
            {
                if let Some((at, pos, face)) =
                    w.clip(old.add_vector(offset), next.add_vector(offset))
                {
                    let d = old.add_vector(offset).distance_squared(at);
                    if d <= best {
                        best = d;
                        end = at.add_vector(Vec3::new(
                            -offset.x + face[0] as f64 * 0.015,
                            -offset.y + face[1] as f64 * 0.015,
                            -offset.z + face[2] as f64 * 0.015,
                        ));
                        block = Some((pos, face));
                    }
                }
            }
            if !q.hit && q.body.phase != f::Phase::Grounded && f::length(q.body.velocity) > 0.1 {
                let mut victim = None;
                let mut nearest = old.distance_squared(end);
                for (who, bounds) in w.targets() {
                    if Some(*who) == p.owner {
                        continue;
                    }
                    for along in [-f::HALF_SEGMENT, 0.0, f::HALF_SEGMENT] {
                        let offset = Vec3::new(axis.x * along, axis.y * along, axis.z * along);
                        if let Some(at) = bounds
                            .inflate(Vec3::new(f::RADIUS, f::RADIUS, f::RADIUS))
                            .clip(old.add_vector(offset), end.add_vector(offset))
                        {
                            let center = at.add_vector(Vec3::new(-offset.x, -offset.y, -offset.z));
                            let d = old.distance_squared(center);
                            if d < nearest {
                                nearest = d;
                                victim = Some((*who, center));
                            }
                        }
                    }
                }
                if let Some((who, at)) = victim {
                    q.hit = true;
                    s.events.push(Event {
                        projectile: p.clone(),
                        hit: Hit::Entity { who, at },
                    });
                    end = at;
                    q.body.phase = f::Phase::Falling;
                    q.body.velocity = [0.0; 3];
                    block = None;
                }
            }
            if let Some((_, face)) = block {
                if face[1] > 0 && !q.landed {
                    q.landed = true;
                    s.cues.push(CueEvent {
                        at: end,
                        kind: Cue::Land,
                    });
                }
                q.body.phase = if face[1] > 0 {
                    f::Phase::Grounded
                } else {
                    f::Phase::Falling
                };
                if face[1] > 0 {
                    q.body.velocity = [0.0;3];
                    p.pitch=0.0; p.previous_pitch=0.0;
                } else {
                    // Slide vertically past doors/walls instead of resetting gravity each tick.
                    for axis in [0,2] { if face[axis] != 0 { q.body.velocity[axis]=0.0; } }
                    let slide=end.add_vector(Vec3::new(0.0,q.body.velocity[1].min(-0.05),0.0));
                    if let Some((at,_,normal))=w.clip(end,slide) {
                        if normal[1]>0 { end=at.add_vector(Vec3::new(0.0,f::RADIUS+0.015,0.0));q.body.phase=f::Phase::Grounded;q.body.velocity=[0.0;3];p.pitch=0.0;p.previous_pitch=0.0; }
                    } else { end=slide; }
                }
            }
            q.body.at = arr(end);
        }
        if q.body.phase == f::Phase::Grounded {
            // Flatten the pose and lower the center onto the highest supporting surface.
            p.pitch=0.0; p.previous_pitch=0.0;
            let center=vec(q.body.at);
            let mut top: Option<f64>=None;
            for x in [-0.4,0.0,0.4] { for z in [-0.4,0.0,0.4] {
                let probe=center.add_vector(Vec3::new(x,0.05,z));
                if let Some((at,_,face))=w.clip(probe,probe.add_vector(Vec3::new(0.0,-0.9,0.0))) {
                    if face[1]>0 { top=Some(top.map_or(at.y,|y|y.max(at.y))); }
                }
            }}
            if let Some(y)=top { q.body.at[1]=y+f::RADIUS+0.015; }
            if !q.landed { q.landed=true; s.cues.push(CueEvent {at:vec(q.body.at),kind:Cue::Land}); }
        }
        p.position = vec(q.body.at);
        p.velocity = vec(q.body.velocity);
        p.stuck = false;
        (true, Hit::None)
    }))
}
