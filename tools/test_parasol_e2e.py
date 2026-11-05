#!/usr/bin/env python3
"""Isolated multiplayer proof of parasol slow fall and fairy cancellation."""
import argparse
import json
import os
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
with tempfile.TemporaryDirectory(prefix='lunatic-parasol-proof-') as temp:
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
  child=launch(d,['--server',f'127.0.0.1:{port}','--name','ParasolProof','--control','--control-socket',str(d/'c.sock'),'--width','640','--height','480'],label)
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
  d=home('client');client,name=connect(d,'kazami_yuuka','yuuka')
  until(lambda:ctl(d,'inventory')['counts'],lambda c:c.get('lunatic:yuuka_parasol')==1)
  console(f'gamemode survival {name}');console(f'clear {name}');console(f'item replace entity {name} weapon.mainhand with lunatic:yuuka_parasol');console(f'tp {name} 0.5 -60 0.5');time.sleep(.6)
  def heard(sound):return ctl(d,'status')['heard'].get(sound,0)
  def flaps():return heard('minecraft:entity.ender_dragon.flap')
  def cancels():return heard('lunatic:entity.fairy.cancel')
  def jump():ctl(d,'key','space','down');time.sleep(.1);ctl(d,'key','space','up')
  def air():console(f'tp {name} 0.5 25 0.5');time.sleep(.25)
  def floating():
   time.sleep(1);v=player(d)['velocity'][1];assert -.65<v<=0,v;return v
  def falling():
   time.sleep(.9);v=player(d)['velocity'][1];assert v<-.5,v;return v
  start=flaps();ctl(d,'mouse','right','down');until(flaps,lambda n:n==start+1,seconds=6);time.sleep(1);ctl(d,'mouse','right','up');assert flaps()==start+1;evidence['held_use_opens_once']=True
  air();evidence['armed_on_ground_then_walk_off']=floating()
  assert not any(e['id']=='slow_falling' for e in player(d)['effects']);evidence['fairy_motion_flag_without_fake_potion']=True
  c=cancels();ctl(d,'mouse','right','click');until(cancels,lambda n:n==c+1,seconds=5);evidence['second_click_cancels']=falling()
  air();f=flaps();ctl(d,'mouse','right','click');until(flaps,lambda n:n==f+1,seconds=5);floating();c=cancels();jump();until(cancels,lambda n:n==c+1,seconds=5);evidence['space_cancels']=falling()
  air();console(f'effect give {name} minecraft:slow_falling 60 0 true');time.sleep(.4);f=flaps();ctl(d,'mouse','right','click');until(flaps,lambda n:n==f+1,seconds=5);c=cancels();jump();until(cancels,lambda n:n==c+1,seconds=5);floating();assert any(e['id']=='slow_falling' for e in player(d)['effects']);evidence['cancel_preserves_slow_falling_potion']=True
  console(f'effect clear {name} minecraft:slow_falling');evidence['potion_removal_restores_normal_gravity']=falling()
  air();f=flaps();ctl(d,'mouse','right','click');until(flaps,lambda n:n==f+1,seconds=5);console(f'item replace entity {name} weapon.mainhand with minecraft:stick');time.sleep(.3);evidence['switching_items_keeps_float']=floating();c=cancels();jump();until(cancels,lambda n:n==c+1,seconds=5);falling();evidence['space_cancel_with_another_item']=True
  console(f'item replace entity {name} weapon.mainhand with minecraft:air');console(f'item replace entity {name} weapon.offhand with lunatic:yuuka_parasol');time.sleep(.4);air();f=flaps();ctl(d,'mouse','right','click');until(flaps,lambda n:n==f+1,seconds=5);floating();c=cancels();jump();until(cancels,lambda n:n==c+1,seconds=5);evidence['offhand_with_empty_mainhand']=True
  console(f'tp {name} 0.5 -60 0.5');client.terminate();client.wait(timeout=10);time.sleep(1)
  client,name=connect(d,'cirno','fairy');console(f'gamemode survival {name}');console(f'clear {name}');console(f'tp {name} 0.5 -60 0.5');time.sleep(.6)
  j=heard('lunatic:entity.fairy.jump');jump();time.sleep(.15);jump();until(lambda:heard('lunatic:entity.fairy.jump'),lambda n:n==j+1,seconds=5);evidence['fairy_air_jump_preserved']=True
  console(f'item replace entity {name} weapon.mainhand with lunatic:yuuka_parasol');time.sleep(.4);air();f=flaps();ctl(d,'mouse','right','click');until(flaps,lambda n:n==f+1,seconds=5);floating();j=heard('lunatic:entity.fairy.jump');c=cancels();jump();until(cancels,lambda n:n==c+1,seconds=5);falling();assert heard('lunatic:entity.fairy.jump')==j;evidence['fairy_space_cancels_without_air_boost']=True
  (a.output/'result.json').write_text(json.dumps(evidence,indent=2)+'\n');print(json.dumps({'passed':True,'checks':evidence}),flush=True)
 finally:
  for child in reversed(children):
   if child.poll() is None:
    try:child.stdin.write(b'stop\n');child.stdin.flush();child.wait(timeout=3)
    except (BrokenPipeError,OSError,subprocess.TimeoutExpired):child.terminate()
  for child in children:
   try:child.wait(timeout=10)
   except subprocess.TimeoutExpired:child.kill();child.wait()
