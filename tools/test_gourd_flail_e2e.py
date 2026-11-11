#!/usr/bin/env python3
"""Isolated multiplayer regression test for the tethered gourd attack."""
import argparse
import json
import os
import re
from pathlib import Path
import shutil
import socket
import subprocess
import tempfile
import time
from test_weapons_e2e import until

p=argparse.ArgumentParser(description=__doc__)
for name in ('game','ctl','resources','mod'):p.add_argument('--'+name,type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args()
for key in ('game','ctl','resources','mod','output'):setattr(a,key,getattr(a,key).resolve())
a.output.mkdir(parents=True,exist_ok=True)
children=[]
with tempfile.TemporaryDirectory(prefix='lunatic-gourd-proof-') as temp:
 root=Path(temp)
 def home(name):
  d=root/name;d.mkdir();(d/'horizon-mods').mkdir();shutil.copy2(a.mod,d/'horizon-mods/lunatic.hmod');(d/'horizon-mods.txt').write_text('mod.lunatic:true\n');(d/'options.txt').write_text('renderDistance:2\nsimulationDistance:2\nautoJump:false\npauseOnLostFocus:false\n');return d
 def launch(d,args,label):
  env=dict(os.environ,HORIZON_HOME=str(d),HORIZON_LAUNCHER_HOME=str(d/'launcher'),XDG_CONFIG_HOME=str(d/'config'),XDG_DATA_HOME=str(d/'data'),XDG_CACHE_HOME=str(d/'cache'),HORIZON_NO_AUDIO='1')
  with (a.output/(label+'.log')).open('w') as log:
   child=subprocess.Popen([str(a.game),'--resources',str(a.resources),'--game-dir',str(d),'--home',str(d),*args],cwd=d,env=env,stdin=subprocess.PIPE,stdout=log,stderr=log)
  children.append(child);return child
 def ctl(d,*args):
  q=subprocess.run([str(a.ctl),'--socket',str(d/'c.sock'),'--timeout','10',*args],capture_output=True,text=True,timeout=15)
  if q.returncode:raise ValueError(q.stdout+q.stderr)
  return json.loads(q.stdout)
 def player(d):return ctl(d,'state')['player']
 def connect(d,char,label):
  (d/'character.json').write_text(json.dumps({'character':'lunatic:'+char}))
  child=launch(d,['--server',f'127.0.0.1:{port}','--name','FlailProof','--control','--control-socket',str(d/'c.sock'),'--width','640','--height','480'],label)
  status=until(lambda:ctl(d,'status'),lambda s:(s.get('server') or {}).get('loaded',False),seconds=90)
  return child,'@a'
 def console(text):server.stdin.write((text+'\n').encode());server.stdin.flush()
 srv=home('server')
 with socket.socket() as s:s.bind(('127.0.0.1',0));port=s.getsockname()[1]
 props={'server-ip':'127.0.0.1','server-port':port,'online-mode':'false','enforce-secure-profile':'false','white-list':'false','view-distance':2,'simulation-distance':2,'spawn-monsters':'false','spawn-animals':'false','level-type':'minecraft:flat','level-seed':123,'generator-settings':json.dumps({'layers':[{'block':'minecraft:bedrock','height':1},{'block':'minecraft:dirt','height':2},{'block':'minecraft:grass_block','height':1}],'biome':'minecraft:plains','features':False,'lakes':False,'structure_overrides':[]})}
 (srv/'server.properties').write_text(''.join(f'{k}={v}\n' for k,v in props.items()));(srv/'eula.txt').write_text('eula=true\n')
 evidence={}
 def oni(d):return next((e for e in player(d)['effects'] if e['id']=='lunatic:oni'),None)
 try:
  server=launch(srv,['--dedicated','--nogui'],'server')
  until(lambda:socket.create_connection(('127.0.0.1',port),1).close(),lambda _:True,seconds=90)
  console('gamerule spawn_mobs false');console('gamerule spawn_monsters false')
  d=home('client');client,name=connect(d,'ibuki_suika','flail')
  until(lambda:ctl(d,'inventory')['counts'],lambda c:bool(c));time.sleep(.4)
  name=re.findall(r'\]: (\S+)\[/127\.0\.0\.1:',(a.output/'server.log').read_text())[0]
  observer=home('observer');watcher,_=connect(observer,'hakurei_reimu','observer')
  watcher_name=re.findall(r'\]: (\S+)\[/127\.0\.0\.1:',(a.output/'server.log').read_text())[-1]
  console(f'gamemode spectator {watcher_name}');console(f'tp {watcher_name} 6.5 -60 0.5 40 10');time.sleep(.4)
  console(f'gamemode survival {name}');console(f'clear {name}');console(f'tp {name} 0.5 -60 0.5 0 -20');console(f'item replace entity {name} weapon.mainhand with lunatic:ibuki_gourd');time.sleep(.5)
  def shots():return [p for p in ctl(d,'status')['projectiles'] if p['kind'].startswith('Thrown')]
  def click():ctl(d,'mouse','left','click')
  def equip(item):console(f'item replace entity {name} weapon.mainhand with {item}');time.sleep(.3)
  def wait_ready():console('tick sprint 180');time.sleep(1)
  ctl(d,'screenshot',str(a.output/'held.png'))
  # Observe outbound travel and a damped reversal before the first terrain impact.
  click();until(shots,bool,seconds=4);samples=[]
  end=time.monotonic()+1.8
  while time.monotonic()<end:
   v=shots()
   if v:samples.append(v[0])
   time.sleep(.025)
  assert samples and any(p['velocity'][2]>0.05 for p in samples),samples
  assert any(p['velocity'][2]<-0.01 for p in samples),samples
  assert all(sum((p['position'][i]-[.82,-58.7,.5][i])**2 for i in range(3))**.5<=8.8 for p in samples),samples
  evidence['momentum_elastic_reversal_and_bounded_range']=True
  resting=until(shots,lambda ps:ps and sum(v*v for v in ps[0]['velocity'])<.0001,seconds=6)[0]
  ctl(d,'look','0','20');time.sleep(.2);ctl(d,'screenshot',str(a.output/'grounded-rope.png'));ctl(d,'look','0','-20')
  console('tick sprint 240');time.sleep(1)
  assert shots() and shots()[0]['id']==resting['id'];evidence['landing_never_auto_retrieves']=True
  assert any(p['id']==resting['id'] for p in ctl(observer,'status')['projectiles'])
  ctl(observer,'look','40','10');ctl(observer,'screenshot',str(a.output/'observer-rope.png'));evidence['second_client_tracks_same_deployed_projectile']=True
  click();until(shots,lambda ps:not ps,seconds=4);assert ctl(d,'inventory')['counts'].get('lunatic:ibuki_gourd')==1;evidence['manual_animated_retrieval_retains_item']=True
  # Early retrieval does not restart/double the four-second launch cooldown.
  wait_ready();click();until(shots,bool,seconds=3);click();until(shots,lambda ps:not ps,seconds=3)
  click();time.sleep(.3);assert not shots();evidence['four_second_attack_cooldown_blocks_early_rethrow']=True
  time.sleep(4);click();until(shots,bool,seconds=3);click();until(shots,lambda ps:not ps,seconds=3)
  evidence['normal_retrieval_does_not_double_cooldown']=True
  # A wall stops outbound flight, but leaves the gourd deployed until clicked.
  wait_ready();console('fill -2 -60 4 2 -55 4 minecraft:stone');ctl(d,'look','0','0');click()
  until(shots,bool,seconds=3);time.sleep(1);assert shots() and shots()[0]['position'][2]<4.1
  click();until(shots,lambda ps:not ps,seconds=3);console('fill -2 -60 4 2 -55 4 minecraft:air');evidence['wall_collision_and_manual_return']=True
  # One mob hit per deployment, modern explosion sound, and the larger punch burst.
  wait_ready();console('summon minecraft:pig 0.5 -60 4 {NoAI:1b,PersistenceRequired:1b}');time.sleep(.4)
  def pig():return next((m for m in ctl(d,'mobs','32')['near'] if m['kind']=='minecraft:pig'),None)
  ctl(d,'look','0','7');before=ctl(d,'status');click()
  hurt=until(pig,lambda m:m is not None and m['health']<10,seconds=4);assert hurt['health']<=4.01,hurt
  assert shots();health=hurt['health'];until(pig,lambda m:m is not None and m['pos'][2]>4.05,seconds=3);time.sleep(.5);assert pig()['health']==health
  after=ctl(d,'status');assert after['heard'].get('lunatic:entity.gourd.impact',0)>before['heard'].get('lunatic:entity.gourd.impact',0)
  assert after['particles'].get('minecraft:explosion',after['particles'].get('explosion',0))>before['particles'].get('minecraft:explosion',before['particles'].get('explosion',0))+7
  evidence['single_hit_damage_sound_particles_and_persistent_deployment']=True
  click();until(shots,lambda ps:not ps,seconds=3);console('kill @e[type=minecraft:pig]')
  # Switching items retrieves automatically and imposes the exceptional eight seconds.
  wait_ready();ctl(d,'look','0','-20');click();until(shots,bool,seconds=3);equip('minecraft:stick');until(shots,lambda ps:not ps,seconds=3)
  equip('lunatic:ibuki_gourd');click();time.sleep(.3);assert not shots();time.sleep(4);click();time.sleep(.3);assert not shots()
  time.sleep(4);click();until(shots,bool,seconds=3);evidence['switch_cleanup_and_doubled_cooldown']=True
  console('kill @e[type=lunatic:gourd_flail]');until(shots,lambda ps:not ps,seconds=4)
  click();time.sleep(.3);assert not shots();evidence['invalid_projectile_cleanup']=True
  console('tick sprint 180');time.sleep(1);click();until(shots,bool,seconds=3)
  console(f'kill {name}');until(shots,lambda ps:not ps,seconds=4);evidence['death_cleans_up_projectile']=True
  (a.output/'result.json').write_text(json.dumps(evidence,indent=2)+'\n');print(json.dumps({'passed':True,'checks':evidence}),flush=True)
 finally:
  for child in reversed(children):
   if child.poll() is None:
    try:child.stdin.write(b'stop\n');child.stdin.flush();child.wait(timeout=3)
    except (BrokenPipeError,OSError,subprocess.TimeoutExpired):child.terminate()
  for child in children:
   try:child.wait(timeout=10)
   except subprocess.TimeoutExpired:child.kill();child.wait()
