#!/usr/bin/env python3
"""Isolated dedicated-server proof of crow tengu gliding and the fan cooldown."""
import argparse
import json
import math
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
with tempfile.TemporaryDirectory(prefix='lunatic-crow-proof-') as temp:
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
  child=launch(d,['--server',f'127.0.0.1:{port}','--name','CrowProof','--control','--control-socket',str(d/'c.sock'),'--width','640','--height','480'],label)
  status=until(lambda:ctl(d,'status'),lambda s:(s.get('server') or {}).get('loaded',False),seconds=90)
  return child,'@a'
 def console(text):server.stdin.write((text+'\n').encode());server.stdin.flush()
 def launches(d):return ctl(d,'status')['heard'].get('minecraft:entity.firework_rocket.launch',0)
 def tap(d):ctl(d,'key','space','down');time.sleep(.08);ctl(d,'key','space','up')
 def ready_to_glide(d,name):
  console(f'gamemode survival {name}');console(f'tp {name} 0.5 120 0.5 0 0');until(lambda:player(d),lambda p:p['position'][1]>110 and not p['on_ground'],seconds=10);ctl(d,'look','0','0');time.sleep(.1);tap(d)
 srv=home('server')
 with socket.socket() as s:s.bind(('127.0.0.1',0));port=s.getsockname()[1]
 props={'server-ip':'127.0.0.1','server-port':port,'online-mode':'false','enforce-secure-profile':'false','white-list':'false','view-distance':2,'simulation-distance':2,'spawn-monsters':'false','spawn-animals':'false','level-type':'minecraft:flat','level-seed':123,'generator-settings':json.dumps({'layers':[{'block':'minecraft:bedrock','height':1},{'block':'minecraft:dirt','height':2},{'block':'minecraft:grass_block','height':1}],'biome':'minecraft:plains','features':False,'lakes':False,'structure_overrides':[]})}
 (srv/'server.properties').write_text(''.join(f'{k}={v}\n' for k,v in props.items()));(srv/'eula.txt').write_text('eula=true\n')
 evidence={}
 try:
  server=launch(srv,['--dedicated','--nogui'],'server')
  until(lambda:socket.create_connection(('127.0.0.1',port),1).close(),lambda _:True,seconds=90)
  d=home('client');client,name=connect(d,'shameimaru_aya','crow')
  until(lambda:ctl(d,'inventory')['counts'],lambda c:bool(c));time.sleep(.3) # Let the initial character kit arrive before clearing it.
  console(f'clear {name}');console(f'item replace entity {name} armor.chest with minecraft:diamond_chestplate');console(f'item replace entity {name} weapon.mainhand with lunatic:hauchiwa_fan')
  until(lambda:ctl(d,'inventory')['counts'],lambda c:c.get('lunatic:hauchiwa_fan')==1)
  ready_to_glide(d,name)
  until(lambda:player(d),lambda p:p['fall_flying'],seconds=8);evidence['glide_with_armor']=True
  console(f'item replace entity {name} armor.chest with minecraft:air');time.sleep(.5);assert player(d)['fall_flying'];evidence['glide_without_chest_item']=True
  console(f'item replace entity {name} armor.chest with minecraft:diamond_chestplate');time.sleep(.3);assert player(d)['fall_flying']
  settled=until(lambda:player(d),lambda p:abs(math.hypot(p['velocity'][0],p['velocity'][2])*20-24)<.3 and p['velocity'][1]<0 and abs(math.hypot(p['velocity'][0],p['velocity'][2])/-p['velocity'][1]-7)<.15,seconds=30)
  evidence['level_flight']={'blocks_per_second':math.hypot(settled['velocity'][0],settled['velocity'][2])*20,'glide_ratio':math.hypot(settled['velocity'][0],settled['velocity'][2])/-settled['velocity'][1]}
  first=launches(d);started=time.monotonic();ctl(d,'use-item')
  until(lambda:launches(d),lambda n:n==first+1,seconds=4);evidence['first_boost']=True
  until(lambda:ctl(d,'status')['rockets'],lambda n:n==0,seconds=3)
  until(lambda:ctl(d,'inventory')['hotbar'][0],lambda s:s.get('damage')==5);evidence['first_boost_costs_five']=True
  console(f'item replace entity {name} weapon.offhand from entity {name} weapon.mainhand');console(f'item replace entity {name} weapon.mainhand with minecraft:air');time.sleep(.4)
  ctl(d,'use-item');time.sleep(.5);assert launches(d)==first+1;evidence['early_repeat_blocked_across_hands']=True
  assert ctl(d,'inventory')['counts'].get('lunatic:hauchiwa_fan')==1;assert ctl(d,'inventory')['offhand']['damage']==5;evidence['cooldown_does_not_damage_fan']=True
  time.sleep(max(0,8.6-(time.monotonic()-started)));assert player(d)['fall_flying'];ctl(d,'use-item')
  until(lambda:launches(d),lambda n:n==first+2,seconds=4);evidence['boost_after_eight_seconds']=True
  until(lambda:ctl(d,'inventory')['offhand'],lambda s:s.get('damage')==10);evidence['offhand_boost_costs_five']=True
  time.sleep(8.6);console(f'item replace entity {name} weapon.offhand with lunatic:hauchiwa_fan[minecraft:damage=245]');until(lambda:ctl(d,'inventory')['offhand'],lambda s:s.get('damage')==245);ctl(d,'use-item');until(lambda:launches(d),lambda n:n==first+3,seconds=4);until(lambda:ctl(d,'inventory')['counts'],lambda c:c.get('lunatic:hauchiwa_fan',0)==0);evidence['fan_breaks_at_250']=True
  ctl(d,'quit');client.wait(timeout=20)
  client,name=connect(d,'hakurei_reimu','non-crow');console(f'clear {name}');console(f'item replace entity {name} armor.chest with minecraft:diamond_chestplate');console(f'item replace entity {name} weapon.mainhand with lunatic:hauchiwa_fan');time.sleep(.5)
  ready_to_glide(d,name);time.sleep(.5);assert not player(d)['fall_flying'];before=launches(d);ctl(d,'use-item');time.sleep(.5);assert launches(d)==before;evidence['non_crow_has_no_virtual_elytra_or_fan_boost']=True
  console(f'item replace entity {name} armor.chest with minecraft:elytra');ready_to_glide(d,name);until(lambda:player(d),lambda p:p['fall_flying'],seconds=8);evidence['ordinary_elytra_still_works']=True
  (a.output/'result.json').write_text(json.dumps(evidence,indent=2)+'\n');print(json.dumps({'passed':True,'checks':evidence}),flush=True)
 finally:
  for child in reversed(children):
   if child.poll() is None:
    try:child.stdin.write(b'stop\n');child.stdin.flush();child.wait(timeout=3)
    except (BrokenPipeError,OSError,subprocess.TimeoutExpired):child.terminate()
  for child in children:
   try:child.wait(timeout=10)
   except subprocess.TimeoutExpired:child.kill();child.wait()
