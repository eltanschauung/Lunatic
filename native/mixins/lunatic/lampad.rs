// SPDX-License-Identifier: GPL-3.0-or-later
use super::Game;
use horizon_client::{projectiles::{Projectile,ProjectileKind,Hit,thrown_type,lampad_motion as motion},mobs::Who,particles::ParticleOptions};
use horizon_core::{geometry::Vec3,random::RandomSource};
use std::{cell::RefCell,collections::HashMap};
const TORCH:&str="lunatic:lampad_torch";
struct Shot{world:u64,dim:String,owner:u128,due:i64,dir:[f64;3],side:bool,color:usize}
struct Zone{world:u64,dim:String,p:Projectile,at:Vec3,born:i64,last:HashMap<Who,i64>}
#[derive(Default)]struct State{side:HashMap<(u64,u128),bool>,shots:Vec<Shot>,zones:Vec<Zone>}
thread_local!{static STATE:RefCell<State>=RefCell::new(State::default());}
impl Game<'_>{
 pub(super)fn lampad_input(&mut self,secondary:bool)->bool{
  if !self.native_on("lunatic")||self.spectator()||self.held().is_none_or(|s|s.item.as_ref()!=TORCH){return false;}
  let mut data=[0;9];data[0]=if secondary{2}else{1};data[1..5].copy_from_slice(&self.yaw.to_be_bytes());data[5..].copy_from_slice(&self.pitch.to_be_bytes());
  let _=self.native_net_send_to_server("lunatic:rod_fire",&data);self.swing_arm();true
 }
 pub(super)fn lampad_fire(&mut self,secondary:bool,dir:[f64;3]){
  if self.spectator()||self.held().is_none_or(|s|s.item.as_ref()!=TORCH)||!self.lunatic_weapon_ready(){return;}
  let world=self.lunatic_world_key();let owner=self.native_acting_id();
  let side=STATE.with_borrow_mut(|s|{let v=s.side.entry((world,owner)).or_default();let old=*v;if !secondary{*v=!*v;}old});
  if secondary {let c=self.world.random.next_int_bound(3).unwrap_or(0)as usize;self.lampad_spawn(true,dir,side,c);}else{
   self.lampad_spawn(false,dir,side,0);
   STATE.with_borrow_mut(|s|for(color,delay)in[(1,3),(2,6)]{s.shots.push(Shot{world,dim:self.dim.id().into(),owner,due:self.world.game_time+delay,dir,side,color});});
  }
  if !self.creative {let held=self.held_mut();if let Some(s)=held {s.damage+=1;if s.damage>=300{*held=None;}}}
 }
 fn lampad_spawn(&mut self,flag:bool,dir:[f64;3],side:bool,color:usize){
  let name=if flag{format!("lunatic:lampad_flag_{}",["red","white","blue"][color])}else{format!("lunatic:lampad_star_{}_{}",if side{"right"}else{"left"},["red","white","blue"][color])};
  let Some(t)=thrown_type(&name)else{return;};let Some((eye,_))=self.aim()else{return;};
  let at=Vec3::new(eye[0]+dir[0]*0.8,eye[1]+dir[1]*0.8,eye[2]+dir[2]*0.8);
  let mut p=Projectile::new(ProjectileKind::Thrown(t),at,Some(Who::Player));p.guest=self.lan_acting;p.no_gravity=true;
  let speed=if flag{1.4}else{1.25};p.velocity=Vec3::new(dir[0]*speed,dir[1]*speed,dir[2]*speed);self.add_projectile(p);self.lunatic_sound("item.lampad_torch.shoot",at,0.7);
 }
 pub(super)fn lampad_pre(&mut self){
  let world=self.lunatic_world_key();let dim=self.dim.id();let now=self.world.game_time;
  let due=STATE.with_borrow_mut(|s|{let(a,b)=std::mem::take(&mut s.shots).into_iter().partition::<Vec<_>,_>(|q|q.world==world&&q.dim==dim&&q.due<=now);s.shots=b;a});
  for q in due{self.lunatic_as_player(q.owner,|g|{let dir=g.aim().map_or(q.dir,|(_,dir)|dir);g.lampad_spawn(false,dir,q.side,q.color)});}
  use std::hash::{Hash,Hasher};let mut h=std::collections::hash_map::DefaultHasher::new();world.hash(&mut h);dim.hash(&mut h);
  motion::begin(h.finish(),&self.creatures.projectiles.iter().filter(|p|motion::ours(p)).map(|p|p.id).collect::<Vec<_>>());
  let zones=STATE.with_borrow_mut(|s|std::mem::take(&mut s.zones));let mut keep=Vec::new();
  for mut z in zones {if z.world!=world||z.dim!=dim{keep.push(z);continue;}let age=now-z.born;if age>=60{continue;}
   let t=age as f64/20.0;let radius=(3.0*t-0.5*t*t).max(1.0);let height=1.0+2.0*t/3.0;
   self.lampad_damage(&z.p,z.at,6.0,radius,height,Some((&mut z.last,now)));
   keep.push(z);
  }
  STATE.with_borrow_mut(|s|s.zones.extend(keep));
 }
 pub(super)fn lampad_post(&mut self){for e in motion::finish(){
  if let Hit::Entity{who,..}=e.hit{self.lampad_direct(&e.projectile,who,e.damage,e.at);}
  if e.flag{
   let mut at=e.at;if let Some(hit)=self.clip_ray([at.x,at.y,at.z],[0.0,-1.0,0.0],40.0){at.y=hit.pos[1]as f64+1.02;}else{at.y=at.y.floor()+1.02;}
   let v=e.projectile.velocity;let n=(v.x*v.x+v.z*v.z).sqrt();let right=if n>1e-6{[v.z/n,0.0,-v.x/n]}else{[1.0,0.0,0.0]};
   for row in 0..9{for col in 0..11{let color=if row<5&&col<6 {if(row+col)%2==0{"white"}else{"blue"}}else if row%2==0{"red"}else{"white"};
    self.lampad_particle(color,false,Vec3::new(at.x+right[0]*col as f64*0.44,at.y+(12-row)as f64*0.44,at.z+right[2]*col as f64*0.44),[0.0;3]);}}
   for i in 0..4{self.lampad_particle("white",false,Vec3::new(at.x,at.y+i as f64*0.44,at.z),[0.0;3]);}
   for i in 0..16{let a=i as f64*std::f64::consts::TAU/16.0;self.lampad_particle("red",true,at,[a.cos()*3.0,0.0,a.sin()*3.0]);}
   let mut z=Zone{world:self.lunatic_world_key(),dim:self.dim.id().into(),p:e.projectile,at,born:self.world.game_time,last:HashMap::new()};
   self.lampad_damage(&z.p,at,6.0,1.0,1.0,Some((&mut z.last,self.world.game_time)));STATE.with_borrow_mut(|s|s.zones.push(z));
  }else{
   self.lampad_damage(&e.projectile,e.at,e.damage,2.0,4.0,None);
   for color in["red","white","blue"]{let mut o=ParticleOptions::named(format!("lunatic:lampad_burst_{color}"));o.scale=0.36;self.lampad_send_particles(&o,None,[e.at.x,e.at.y,e.at.z],[0.4;3],0.1,8);}
  }
  self.lunatic_sound("entity.yin_yang_orb.impact",e.at,0.6);
 }}
 fn lampad_send_particles(&mut self,o:&ParticleOptions,_state:Option<horizon_world::registry::BlockStateId>,at:[f64;3],spread:[f64;3],speed:f64,count:i32){
  self.lan_send_particles(super::lan_particles::QueuedParticle{dimension:self.dim.id(),options:o.clone(),state:None,at,spread,speed:[speed;3],count,randomization:0});
 }
 fn lampad_particle(&mut self,color:&str,ring:bool,at:Vec3,velocity:[f64;3]){let o=ParticleOptions::named(format!("lunatic:lampad_{}_{color}",if ring{"ring"}else{"flag"}));self.lampad_send_particles(&o,None,[at.x,at.y,at.z],velocity,1.0,0);}
 pub(super) fn lampad_direct(&mut self,p:&Projectile,who:Who,amount:f32,at:Vec3){
  let mut shot=p.clone();if let ProjectileKind::Thrown(t)=p.kind{shot.kind=ProjectileKind::Thrown(horizon_client::projectiles::lunatic_motion::lampad_damage_type(t,amount));}
  let hit=Hit::Entity{who,at};
  if matches!(who,Who::Mob(_)) {if let Some(conn)=shot.guest {
   let uuid=self.lan.as_ref().and_then(|h|h.hub.guest(conn)).map(|g|g.uuid);
   if let Some(uuid)=uuid{self.lunatic_as_player(uuid,|g|{let _=g.projectile_hit(&shot,hit);});}
  }else{let _=self.projectile_hit(&shot,hit);}}
  else if let Some(v)=self.projectile_pvp_victim(&shot,hit){let _=self.projectile_hits_player(&shot,hit,v);}
 }
 fn lampad_damage(&mut self,p:&Projectile,at:Vec3,amount:f32,radius:f64,height:f64,mut pulse:Option<(&mut HashMap<Who,i64>,i64)>){
  let mut victims=Vec::new();for m in &self.mobs{let b=m.bounds();if m.alive()&&m.position.distance_squared(at)<=radius*radius&&b.max(horizon_core::geometry::Axis::Y)>=at.y-height/2.0&&b.min(horizon_core::geometry::Axis::Y)<=at.y+height/2.0{victims.push((Who::Mob(m.id),b.center()));}}
  for(w,b)in self.lan_projectile_targets(){if b.center().distance_squared(at)<=radius*radius&&b.max(horizon_core::geometry::Axis::Y)>=at.y-height/2.0&&b.min(horizon_core::geometry::Axis::Y)<=at.y+height/2.0{victims.push((w,b.center()));}}
  for(w,point)in victims{if let Some((last,now))=&mut pulse{if last.get(&w).is_some_and(|t|*now-*t<10){continue;}last.insert(w,*now);}self.lampad_direct(p,w,amount,point);}
 }
 pub(super)fn lampad_stop(&self){STATE.with_borrow_mut(|s|*s=State::default());motion::reset();}
}
