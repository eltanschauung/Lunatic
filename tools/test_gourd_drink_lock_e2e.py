#!/usr/bin/env python3
"""Reject drinking during gourd flight/return while preserving held-item drinking."""
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
  d=root/name;d.mkdir();(d/'horizon-mods').mkdir();shutil.copy2(a.mod,d/'horizon-mods/lunatic.hmod');(d/'horizon-mods.txt').write_text('mod.lunatic:true\nmod.voicechat:false\nmod.voicechat_server:false\nmod.firstpersonmodel_horizon:false\n');(d/'options.txt').write_text('renderDistance:2\nsimulationDistance:2\nautoJump:false\npauseOnLostFocus:false\n');return d
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
 def oni(d):return next((e for e in player(d)['effects'] if e['id']=='lunatic:oni'),None)
 try:
  server=launch(srv,['--dedicated','--nogui'],'server')
  until(lambda:socket.create_connection(('127.0.0.1',port),1).close(),lambda _:True,seconds=90)
  console('gamerule spawn_mobs false');console('gamerule spawn_monsters false')
  d=home('client');client,name=connect(d,'ibuki_suika','flail')
  until(lambda:ctl(d,'inventory')['counts'],lambda c:bool(c));time.sleep(.4)
  assert player(d)['food']==30,player(d)
  evidence['oni_hunger_capacity_30']=True
  name=re.findall(r'\]: (\S+)\[/127\.0\.0\.1:',(a.output/'server.log').read_text())[0]
  observer=home('observer');watcher,_=connect(observer,'hakurei_reimu','observer')
  watcher_name=re.findall(r'\]: (\S+)\[/127\.0\.0\.1:',(a.output/'server.log').read_text())[-1]
  console(f'gamemode spectator {watcher_name}');console(f'tp {watcher_name} 6.5 -60 0.5 40 10');time.sleep(.4)
  console(f'gamemode survival {name}');console(f'clear {name}');console(f'tp {name} 0.5 -60 0.5 0 0');console(f'item replace entity {name} weapon.mainhand with lunatic:ibuki_gourd');time.sleep(.5)
  def shots():return [p for p in ctl(d,'status')['projectiles'] if p['kind'].startswith('Thrown')]
  def click():ctl(d,'mouse','left','click')
  def equip(item):console(f'item replace entity {name} weapon.mainhand with {item}');time.sleep(.3)
  def wait_ready():console('tick sprint 180');time.sleep(1)
  def try_drink(blocked):
   ctl(d,'mouse','right','down')
   try:
    time.sleep(.15);using=ctl(d,'state')['using']
    assert (using is None)==blocked,{'blocked':blocked,'using':using}
   finally:ctl(d,'mouse','right','up')
  try_drink(False);evidence['held_gourd_can_drink']=True
  click();until(shots,bool,seconds=4);time.sleep(.15)
  try_drink(True);evidence['outbound_gourd_cannot_drink']=True
  until(shots,lambda ps:bool(ps) and sum(v*v for v in ps[0]['velocity'])<.0001,seconds=6)
  try_drink(True);evidence['landed_gourd_cannot_drink']=True
  click();time.sleep(.1)
  if shots():try_drink(True)
  until(shots,lambda ps:not ps,seconds=4)
  try_drink(False);evidence['returned_gourd_can_drink']=True
  (a.output/'result.json').write_text(json.dumps(evidence,indent=2)+'\n');print(json.dumps({'passed':True,'checks':evidence}),flush=True)

 finally:
  for child in reversed(children):
   if child.poll() is None:
    try:child.stdin.write(b'stop\n');child.stdin.flush();child.wait(timeout=3)
    except (BrokenPipeError,OSError,subprocess.TimeoutExpired):child.terminate()
  for child in children:
   try:child.wait(timeout=10)
   except subprocess.TimeoutExpired:child.kill();child.wait()
