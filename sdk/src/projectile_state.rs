// SPDX-License-Identifier: GPL-3.0-or-later
//! The two replacement crates share projectile state by owning it in the mod library.
use crate::gourd_physics::Body;
use std::{any::Any, cell::RefCell, collections::{HashMap, HashSet}, thread::LocalKey, time::Instant};

type Slot = LocalKey<RefCell<Option<Box<dyn Any>>>>;

thread_local! {
    pub static ORBS: RefCell<Option<Box<dyn Any>>> = const { RefCell::new(None) };
    pub static GOURD: RefCell<Option<Box<dyn Any>>> = const { RefCell::new(None) };
    pub static LAMPAD: RefCell<Option<Box<dyn Any>>> = const { RefCell::new(None) };
}

pub fn with_state<T: Default + 'static, R>(slot: &'static Slot, f: impl FnOnce(&mut T) -> R) -> R {
    slot.with_borrow_mut(|state| {
        let state = state.get_or_insert_with(|| Box::new(T::default()));
        f(state.downcast_mut::<T>().expect("Lunatic projectile state has a shared type"))
    })
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OrbSound { Bounce, Impact, CloseDetonation }
pub struct OrbFlight<W> {
    pub bounces: u8,
    pub target: Option<W>,
    pub seen: Instant,
}
#[derive(Clone)]
pub struct OrbEvent<P, V> {
    pub sound: OrbSound,
    pub projectile: P,
    pub position: V,
    pub normal: [i8; 3],
}
pub struct OrbState<P, V, W> {
    pub level: u64,
    pub server: bool,
    pub flights: HashMap<(u64, u32), OrbFlight<W>>,
    pub events: Vec<OrbEvent<P, V>>,
    pub live: HashSet<u32>,
}
impl<P, V, W> Default for OrbState<P, V, W> {
    fn default() -> Self {
        Self { level: 0, server: false, flights: HashMap::new(), events: Vec::new(), live: HashSet::new() }
    }
}

pub struct GourdFlight {
    pub body: Body,
    pub anchor: [f64; 3],
    pub owner: u128,
    pub hit: bool,
    pub landed: bool,
}
#[derive(Clone)]
pub struct GourdEvent<P, H> {
    pub projectile: P,
    pub hit: H,
}
#[derive(Clone, Copy)]
pub enum Cue { Elastic, Land }
pub struct GourdCue<V> { pub at: V, pub kind: Cue }
pub struct GourdState<P, H, V> {
    pub level: u64,
    pub flights: HashMap<(u64, u32), GourdFlight>,
    pub events: Vec<GourdEvent<P, H>>,
    pub cues: Vec<GourdCue<V>>,
}
impl<P, H, V> Default for GourdState<P, H, V> {
    fn default() -> Self {
        Self { level: 0, flights: HashMap::new(), events: Vec::new(), cues: Vec::new() }
    }
}

#[derive(Clone)]
pub struct LampadEvent<P, V, H> {
    pub projectile: P,
    pub at: V,
    pub flag: bool,
    pub damage: f32,
    pub hit: H,
}
pub struct LampadFlight<V> { pub origin: V, pub forward: V, pub side: V }
pub struct LampadState<P, V, H> {
    pub level: u64,
    pub flights: HashMap<(u64, u32), LampadFlight<V>>,
    pub events: Vec<LampadEvent<P, V, H>>,
}
impl<P, V, H> Default for LampadState<P, V, H> {
    fn default() -> Self {
        Self { level: 0, flights: HashMap::new(), events: Vec::new() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replacement_calls_share_only_the_current_threads_projectile_state() {
        type State = OrbState<u32, [f64; 3], u32>;
        with_state::<State, _>(&ORBS, |state| {
            state.level = 12;
            state.events.push(OrbEvent {
                sound: OrbSound::Impact,
                projectile: 27,
                position: [0.0, 1.0, 2.0],
                normal: [0, 1, 0],
            });
        });
        std::thread::spawn(|| {
            with_state::<State, _>(&ORBS, |state| assert!(state.events.is_empty()));
        })
        .join()
        .unwrap();
        with_state::<State, _>(&ORBS, |state| {
            assert_eq!(state.level, 12);
            assert_eq!(state.events.pop().unwrap().projectile, 27);
        });
    }
}
