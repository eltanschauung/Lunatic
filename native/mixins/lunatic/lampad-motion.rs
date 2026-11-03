// SPDX-License-Identifier: GPL-3.0-or-later
use super::{Projectile,ProjectileWorld,Hit};
use horizon_core::geometry::Vec3;
use std::{cell::RefCell,collections::HashMap};
#[derive(Clone)] pub struct Event {pub projectile:Projectile,pub at:Vec3,pub flag:bool,pub damage:f32,pub hit:Hit}
struct Flight {origin:Vec3,forward:Vec3,side:Vec3}
#[derive(Default)] struct State {level:u64,flights:HashMap<(u64,u32),Flight>,events:Vec<Event>}
thread_local!{static STATE:RefCell<State>=RefCell::new(State::default());}
pub fn ours(p:&Projectile)->bool {p.kind.id().starts_with("lunatic:lampad_star_")||p.kind.id().starts_with("lunatic:lampad_flag_")}
pub fn begin(level:u64,live:&[u32]) {STATE.with_borrow_mut(|s|{s.level=level;s.flights.retain(|(l,id),_|*l!=level||live.contains(id));});}
pub fn finish()->Vec<Event>{STATE.with_borrow_mut(|s|std::mem::take(&mut s.events))}
pub fn reset(){STATE.with_borrow_mut(|s|*s=State::default());}
fn add(a:Vec3,b:Vec3)->Vec3{Vec3::new(a.x+b.x,a.y+b.y,a.z+b.z)}
fn mul(a:Vec3,n:f64)->Vec3{Vec3::new(a.x*n,a.y*n,a.z*n)}
fn sub(a:Vec3,b:Vec3)->Vec3{add(a,mul(b,-1.0))}
fn unit(v:Vec3)->Vec3{let n=(v.x*v.x+v.y*v.y+v.z*v.z).sqrt();if n<1e-9 {Vec3::new(0.0,0.0,1.0)}else{mul(v,1.0/n)}}
pub fn tick(p:&mut Projectile,w:&dyn ProjectileWorld)->Option<(bool,Hit)>{
 if !ours(p){return None;}
 Some(STATE.with_borrow_mut(|s|{
  let key=(s.level,p.id);let flag=p.kind.id().contains("lampad_flag_");
  if p.age==0||!s.flights.contains_key(&key){let f=unit(p.velocity);let side=if f.x*f.x+f.z*f.z>1e-8 {unit(Vec3::new(f.z,0.0,-f.x))}else{Vec3::new(0.0,-f.z,f.y)};
   s.flights.insert(key,Flight{origin:p.position,forward:f,side});}
  p.previous=p.position;p.previous_yaw=p.yaw;p.previous_pitch=p.pitch;p.age+=1;
  let f=&s.flights[&key];let distance=(p.age as f64*1.25).min(33.0);
  let next=if flag {p.velocity=mul(p.velocity,0.93);p.velocity.y-=22.0/400.0;add(p.position,p.velocity)}else{
   let sign=if p.kind.id().contains("_left_"){-1.0}else{1.0};
   let bend=((6.0*distance/33.0).exp()-1.0)/(6.0_f64.exp()-1.0)*33.0*sign;
   let n=add(f.origin,add(mul(f.forward,distance),mul(f.side,bend)));p.velocity=sub(n,p.position);n};
  let block=w.clip(p.position,next);let end=block.map_or(next,|b|b.0);
  let mut hit=block.map_or(Hit::None,|(at,pos,face)|Hit::Block{at,pos,face});let mut best=p.position.distance_squared(end);
  for (who,b)in w.targets(){if Some(*who)==p.owner {continue;}if let Some(at)=b.clip(p.position,end){let d=p.position.distance_squared(at);if d<=best{best=d;hit=Hit::Entity{who:*who,at};}}}
  p.position=match hit{Hit::Entity{at,..}|Hit::Block{at,..}=>at,Hit::None=>next};p.left_owner=true;p.needs_sync=true;
  let expired=if flag {p.age>=18+p.id%9||p.velocity.distance_squared(Vec3::ZERO)<0.000625}else{distance>=33.0||p.age>=120};
  if expired||!matches!(hit,Hit::None){let damage=if flag{6.0}else{6.0+(3.0*p.age as f32/120.0).floor()};
   s.events.push(Event{projectile:p.clone(),at:p.position,flag,damage,hit});s.flights.remove(&key);return(false,Hit::None);}
  (true,Hit::None)
 }))
}
