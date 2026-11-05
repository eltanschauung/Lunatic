#!/usr/bin/env python3
"""Isolated multiplayer proof of Oni/Yuuka melee traits and intrinsic parasol enchantments."""
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
with tempfile.TemporaryDirectory(prefix='lunatic-melee-proof-') as temp:
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
  child=launch(d,['--server',f'127.0.0.1:{port}','--name','MeleeProof','--control','--control-socket',str(d/'c.sock'),'--width','640','--height','480'],label)
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
  d=home('client');client,name=connect(d,'ibuki_suika','oni')
  console(f'gamemode survival {name}');console(f'tp {name} 0.5 -60 0.5 0 0');console(f'clear {name}');time.sleep(.5)
  def mob():return next((m for m in ctl(d,'mobs','32')['near'] if m['kind']=='minecraft:pig'),None)
  def explosions():return ctl(d,'status')['heard'].get('lunatic:entity.explosion_old',0)
  def hit(expected, pop, invulnerable=False):
   console('kill @e[type=minecraft:pig]');time.sleep(.3)
   tag='{Silent:1b,PersistenceRequired:1b,Health:100f,attributes:[{id:"minecraft:max_health",base:100.0},{id:"minecraft:movement_speed",base:0.0}]'+(',Invulnerable:1b}' if invulnerable else '}')
   console('summon minecraft:pig 0.5 -60 2.5 '+tag)
   m=until(mob,lambda m:m is not None and m['health']==100,seconds=6)
   console(f'tp {name} 0.5 -60 0.5 0 0');time.sleep(1.2)
   heard=explosions();ctl(d,'attack',str(m['id']))
   if not invulnerable:until(mob,lambda m:m is not None and m['health']<100,seconds=5)
   time.sleep(.7);m=mob();damage=100-m['health'];assert abs(damage-expected)<.02,(expected,m)
   assert explosions()==heard+int(pop),(pop,heard,explosions())
   return {'damage':damage,'displacement':m['pos'][2]-2.5,'pop':pop}
  def equip(item):
   console(f'item replace entity {name} weapon.mainhand with {item}');time.sleep(.35)
  equip('minecraft:air');evidence['normal_fist']=hit(1,False)
  console(f'effect give {name} lunatic:oni 600 0 true');until(lambda:oni(d),lambda e:e is not None,seconds=5)
  evidence['oni_fist_strength_ii']=hit(7,True)
  evidence['invulnerable_punch_has_no_pop']=hit(0,False,True)
  equip('minecraft:iron_sword');evidence['oni_weapon_strength_i']=hit(9,False)
  equip('minecraft:stick');evidence['held_item_is_not_a_fist']=hit(4,False)
  console(f'effect clear {name}');until(lambda:oni(d),lambda e:e is None,seconds=5)
  equip('minecraft:air');evidence['oni_clear_restores_baseline']=hit(1,False)
  # Reconnect the same installation with a different catalog character.
  client.terminate();client.wait(timeout=10);time.sleep(1)
  client,name=connect(d,'kazami_yuuka','yuuka');console(f'tp {name} 0.5 -60 0.5 0 0');console(f'clear {name}');time.sleep(.5)
  equip('minecraft:air');evidence['yuuka_fist_strength_i']=hit(4,True)
  equip('minecraft:iron_sword');evidence['yuuka_weapon_strength_i']=hit(9,False)
  equip('lunatic:yuuka_parasol');inv=ctl(d,'inventory');assert 'minecraft:knockback' in json.dumps(inv)
  evidence['parasol_iron_sword_and_knockback_iii']=hit(9,False)
  equip('minecraft:iron_sword[minecraft:enchantments={"minecraft:knockback":3}]');evidence['parasol_knockback_reference']=hit(9,False)
  assert abs(evidence['parasol_iron_sword_and_knockback_iii']['displacement']-evidence['parasol_knockback_reference']['displacement'])<.7
  assert evidence['yuuka_fist_strength_i']['displacement']>.5
  assert evidence['oni_fist_strength_ii']['displacement']>evidence['yuuka_fist_strength_i']['displacement']
  # Explicit component removal on creation must not produce an unenchanted parasol.
  equip('lunatic:yuuka_parasol[!minecraft:enchantments]');evidence['parasol_explicit_removal_is_normalized']=hit(9,False)
  assert abs(evidence['parasol_explicit_removal_is_normalized']['displacement']-evidence['parasol_knockback_reference']['displacement'])<.7
  console(f'effect give {name} lunatic:oni 600 0 true');until(lambda:oni(d),lambda e:e is not None,seconds=5)
  equip('minecraft:air');evidence['oni_yuuka_fist_does_not_stack']=hit(7,True)
  equip('minecraft:iron_sword');evidence['oni_yuuka_weapon_does_not_stack']=hit(9,False)
  console(f'effect give {name} minecraft:strength 600 1 true');time.sleep(.5)
  evidence['stronger_vanilla_strength_wins']=hit(12,False)
  ctl(d,'screenshot',str(a.output/'yuuka-melee.png'))
  (a.output/'result.json').write_text(json.dumps(evidence,indent=2)+'\n');print(json.dumps({'passed':True,'checks':evidence}),flush=True)
 finally:
  for child in reversed(children):
   if child.poll() is None:
    try:child.stdin.write(b'stop\n');child.stdin.flush();child.wait(timeout=3)
    except (BrokenPipeError,OSError,subprocess.TimeoutExpired):child.terminate()
  for child in children:
   try:child.wait(timeout=10)
   except subprocess.TimeoutExpired:child.kill();child.wait()
