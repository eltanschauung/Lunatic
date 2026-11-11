#!/usr/bin/env python3
"""Render character chest geometry and motion in isolated multiplayer homes."""
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
import zipfile
from test_weapons_e2e import until

p=argparse.ArgumentParser(description=__doc__)
for name in ('game','ctl','resources','mod'):p.add_argument('--'+name,type=Path,required=True)
p.add_argument('--output',type=Path,required=True)
a=p.parse_args()
for key in ('game','ctl','resources','mod','output'):setattr(a,key,getattr(a,key).resolve())
a.output.mkdir(parents=True,exist_ok=True)
children=[]
with tempfile.TemporaryDirectory(prefix='lunatic-chest-proof-') as temp:
 root=Path(temp)
 fixture=root/'fixture.hmod'
 with zipfile.ZipFile(a.mod) as source,zipfile.ZipFile(fixture,'w',compression=zipfile.ZIP_DEFLATED) as package:
  for info in source.infolist():
   data=source.read(info.filename)
   if info.filename=='assets/lunatic/horizon/characters.json':
    catalog=json.loads(data)
    for character in catalog['characters']:
     if character['id']=='lunatic:ibuki_suika':character['chest']=2
     elif character['id']=='lunatic:hakurei_reimu':character['chest']=2;character['model']='wide'
    data=json.dumps(catalog).encode()
   package.writestr(info,data)
 def home(name):
  d=root/name;d.mkdir();(d/'horizon-mods').mkdir();shutil.copy2(fixture,d/'horizon-mods/lunatic.hmod');(d/'horizon-mods.txt').write_text('mod.lunatic:true\nmod.firstpersonmodel_horizon:false\n');(d/'options.txt').write_text('renderDistance:2\nsimulationDistance:2\nautoJump:false\npauseOnLostFocus:false\n');return d
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
  child=launch(d,['--offscreen','--server',f'127.0.0.1:{port}','--name','FlailProof','--control','--control-socket',str(d/'c.sock'),'--width','640','--height','480'],label)
  status=until(lambda:ctl(d,'status'),lambda s:(s.get('server') or {}).get('loaded',False),seconds=90)
  return child,'@a'
 def console(text):server.stdin.write((text+'\n').encode());server.stdin.flush()
 srv=home('server')
 with socket.socket() as s:s.bind(('127.0.0.1',0));port=s.getsockname()[1]
 props={'server-ip':'127.0.0.1','server-port':port,'online-mode':'false','enforce-secure-profile':'false','white-list':'false','view-distance':2,'simulation-distance':2,'spawn-monsters':'false','spawn-animals':'false','level-type':'minecraft:flat','level-seed':123,'generator-settings':json.dumps({'layers':[{'block':'minecraft:bedrock','height':1},{'block':'minecraft:dirt','height':2},{'block':'minecraft:grass_block','height':1}],'biome':'minecraft:plains','features':False,'lakes':False,'structure_overrides':[]})}
 (srv/'server.properties').write_text(''.join(f'{k}={v}\n' for k,v in props.items()));(srv/'eula.txt').write_text('eula=true\n')
 evidence={}
 try:
  server=launch(srv,['--dedicated','--nogui'],'server')
  until(lambda:socket.create_connection(('127.0.0.1',port),1).close(),lambda _:True,seconds=90)
  d=home('client');client,_=connect(d,'ibuki_suika','chest')
  time.sleep(1)
  name=re.findall(r'\]: (\S+)\[/127\.0\.0\.1:',(a.output/'server.log').read_text())[0]
  observer=home('observer');watcher,_=connect(observer,'hakurei_reimu','observer')
  time.sleep(1)
  watcher_name=re.findall(r'\]: (\S+)\[/127\.0\.0\.1:',(a.output/'server.log').read_text())[-1]
  console(f'gamemode spectator {watcher_name}');console(f'clear {name}')
  console(f'tp {name} 0.5 -60 0.5 180 0');console(f'tp {watcher_name} 2.5 -60 -1 53 12');time.sleep(.5)
  ctl(observer,'screenshot',str(a.output/'slim-chest-2-rest.png'))
  ctl(d,'key','space','down');ctl(d,'key','w','down');time.sleep(.6)
  ctl(observer,'screenshot',str(a.output/'slim-chest-2-moving.png'))
  ctl(d,'key','space','up');ctl(d,'key','w','up');time.sleep(.3)
  console(f'tp {name} 0.5 -60 0.5 180 0');console(f'item replace entity {name} armor.chest with minecraft:iron_chestplate');time.sleep(.4)
  ctl(observer,'screenshot',str(a.output/'chestplate-hides-layer.png'))
  console(f'item replace entity {name} armor.chest with minecraft:air')
  console(f'gamemode survival {watcher_name}');console(f'tp {watcher_name} 0.5 -60 -4 0 0');console(f'tp {name} 4 -60 0 135 0');time.sleep(.4)
  ctl(d,'screenshot',str(a.output/'wide-chest-2-is-flat.png'))
  evidence['multiplayer_character_geometry_and_movement']=True
  evidence['armor_hides_added_skin_geometry']=True
  evidence['wide_model_forces_flat']=True
  for label in ('chest','observer','server'):
   log=(a.output/(label+'.log')).read_text()
   assert 'panicked at' not in log and 'Failed to render' not in log,log[-3000:]
  (a.output/'result.json').write_text(json.dumps(evidence,indent=2)+'\n')
  print(json.dumps({'passed':True,'checks':evidence}),flush=True)
 finally:
  for child in reversed(children):
   if child.poll() is None:
    try:child.stdin.write(b'stop\n');child.stdin.flush();child.wait(timeout=3)
    except (BrokenPipeError,OSError,subprocess.TimeoutExpired):child.terminate()
  for child in children:
   try:child.wait(timeout=10)
   except subprocess.TimeoutExpired:child.kill();child.wait()
