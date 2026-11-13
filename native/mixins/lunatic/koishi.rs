// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_client::{mobs::Mob, particles::ParticleOptions};
use horizon_core::random::RandomSource;
use std::cell::RefCell;
const KNIFE: &str = "lunatic:koishi_knife";
#[derive(Clone)] struct Hit { dimension: String, id: u32, ticks: i32 }
#[derive(Clone)] struct Tick { dimension: String, id: Option<i32>, owner: Option<u128>, hurt: bool }
thread_local! {
 static HITS: RefCell<Vec<Hit>> = const {RefCell::new(Vec::new())};
 static TICKS: RefCell<Vec<Tick>> = const {RefCell::new(Vec::new())};
}
impl Game<'_> {
 pub(super) fn koishi_concealed(&self, uuid:u128)->bool {
  self.native_on("lunatic") && self.character_for(uuid).is_some_and(|c| c.has_race("koishi"))
   && self.native_effects(uuid).is_none_or(|e|!e.iter().any(|e|e.id=="lunatic:revealed" && e.duration!=0))
 }
 pub(super) fn koishi_opacity(&self,m:&Mob,camera:glam::DVec3)->f32 {
  if !self.native_on("lunatic") || m.kind.id!="minecraft:player" {return 1.0;}
  let local = self.player_model.as_ref().is_some_and(|own| own.id == m.id);
  let Some(c)=self.remote_player_uuid(m.id).and_then(|u|self.character_for(u)).or_else(||local.then_some(self.character.as_ref()).flatten()) else{return 1.0;};
  if !c.has_race("koishi") {return 1.0;}
  let viewer=self.remote.as_ref().and_then(|r|self.character_for(r.profile())).or(self.character.as_ref());
  let exempt=viewer.is_some_and(|c|c.has_race("satori")||c.has_race("koishi")||c.id.contains("satori")||c.id.contains("koishi"));
  let from=self.player.as_ref().map_or(camera,|p|glam::DVec3::new(p.position.x,p.position.y,p.position.z));
  let at=glam::DVec3::new(m.position.x,m.position.y,m.position.z);
  super::koishi_policy::opacity(from.distance(at),exempt,m.has_effect("lunatic:revealed"))
 }
 pub(super) fn koishi_knife_hit(&mut self,id:u32) {
  if self.remote.is_some()||!self.native_on("lunatic")||!self.held().is_some_and(|s|s.item.as_ref()==KNIFE) {return;}
  if self.world.random.next_int_bound(4).unwrap_or(1)!=0 {return;}
  let ticks=20*(2+self.world.random.next_int_bound(3).unwrap_or(0));
  HITS.with_borrow_mut(|q|q.push(Hit{dimension:self.dim.id().into(),id,ticks}));
 }
 pub(super) fn koishi_bleed_tick(&self,dimension:&str,id:Option<i32>, ticks:i32) {
  TICKS.with_borrow_mut(|q|q.push(Tick{dimension:dimension.into(),id,owner:self.lan_acting_uuid,hurt:ticks%20==0}));
 }
 pub(super) fn koishi_stop(&self) {HITS.with_borrow_mut(Vec::clear);TICKS.with_borrow_mut(Vec::clear);}
 pub(super) fn koishi_post(&mut self) {
  if self.remote.is_some()||!self.native_on("lunatic") {return;}
  let dim=self.dim.id();
  let players=self.native_server_players();
  let hits=HITS.with_borrow_mut(|q| {let (mine,other):(Vec<_>,Vec<_>)=q.drain(..).partition(|h|h.dimension==dim);*q=other;mine});
  for hit in hits {
   if let Some(p)=players.iter().find(|p|p.entity_id==hit.id as i32 && p.dimension==dim) {
    self.native_add_effect(p.uuid,&super::native_player_state::NativeEffect{id:"lunatic:bleeding".into(),duration:hit.ticks,amplifier:0,ambient:false,particles:false});
   } else if let Some(m)=self.mobs.iter_mut().find(|m|m.id==hit.id) {m.add_effect("lunatic:bleeding",hit.ticks,0);}
  }
  let ticks=TICKS.with_borrow_mut(|q|{let (mine,other):(Vec<_>,Vec<_>)=q.drain(..).partition(|t|t.dimension==dim);*q=other;mine});
  for t in ticks {
   let target=t.id.map(|id|id as u32).or_else(||players.iter().find(|p|Some(p.uuid)==t.owner).map(|p|p.entity_id as u32));
   let Some(id)=target else{continue;};
   let at=players.iter().find(|p|p.entity_id==id as i32).map(|p|p.position).or_else(||self.mobs.iter().find(|m|m.id==id).map(|m|[m.position.x,m.position.y,m.position.z]));
   if t.hurt {self.native_hurt_entity(dim,id,1.0,"minecraft:magic",None);}
   if let Some(mut at)=at {
    at[0]+=(self.world.random.next_int_bound(41).unwrap_or(20) as f64-20.0)/100.0;
    at[1]+=0.6+self.world.random.next_int_bound(70).unwrap_or(35) as f64/100.0;
    at[2]+=(self.world.random.next_int_bound(41).unwrap_or(20) as f64-20.0)/100.0;
    let mut options=ParticleOptions::dust([0.55,0.015,0.035],0.6);
    options.color=[0.55,0.015,0.035,1.0];
    self.creatures.lan_particles.push(super::lan_particles::QueuedParticle {
     dimension:dim, options, state:None, at, spread:[0.0,-1.0,0.0], speed:[0.035;3], count:0, randomization:0
    });
   }
  }
 }
}
