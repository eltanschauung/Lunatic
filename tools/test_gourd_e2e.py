#!/usr/bin/env python3
"""Isolated multiplayer proof of the Ibuki Gourd and Oni melee effect."""
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
  child=launch(d,['--server',f'127.0.0.1:{port}','--name','GourdProof','--control','--control-socket',str(d/'c.sock'),'--width','640','--height','480'],label)
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
  d=home('client');client,name=connect(d,'ibuki_suika','gourd')
  until(lambda:ctl(d,'inventory')['counts'],lambda c:c.get('lunatic:ibuki_gourd')==1);evidence['suika_starting_gourd']=True
  console(f'gamemode survival {name}');console(f'tp {name} 0.5 -60 0.5 0 0');console(f'clear {name}');console(f'item replace entity {name} weapon.mainhand with lunatic:ibuki_gourd');time.sleep(.5)
  dragon_before=ctl(d,'status')['heard'].get('minecraft:entity.ender_dragon.ambient',0)
  ctl(d,'mouse','right','down');until(lambda:ctl(d,'status')['using'],lambda x:x is not None,seconds=4);time.sleep(.4);assert oni(d) is None;ctl(d,'mouse','right','up');until(lambda:ctl(d,'status')['using'],lambda x:x is None,seconds=4);assert oni(d) is None;evidence['interrupted_drink_grants_nothing']=True
  assert ctl(d,'status')['heard'].get('minecraft:entity.ender_dragon.ambient',0)==dragon_before;evidence['cancelled_drink_has_no_dragon_sound']=True
  ctl(d,'mouse','right','down');time.sleep(.5);assert oni(d) is None;evidence['effect_waits_for_completion']=True
  e=until(lambda:oni(d),lambda x:x is not None,seconds=4);ctl(d,'mouse','right','up');assert 235<=e['duration_ticks']<=400 and e['amplifier']==0;evidence['completed_drink_grants_random_twelve_to_twenty_seconds']=True
  until(lambda:ctl(d,'status')['heard'].get('minecraft:entity.ender_dragon.ambient',0),lambda n:n==dragon_before+1,seconds=4);time.sleep(.3);assert ctl(d,'status')['heard'].get('minecraft:entity.ender_dragon.ambient',0)==dragon_before+1;evidence['oni_application_plays_one_dragon_sound']=True
  assert ctl(d,'inventory')['counts'].get('lunatic:ibuki_gourd')==1;evidence['gourd_retained_without_bottle']=True;assert not ctl(d,'inventory')['counts'].get('minecraft:glass_bottle',0)
  assert ctl(d,'status')['heard'].get('minecraft:entity.generic.drink',0)>0;evidence['standard_drinking_sound']=True
  ctl(d,'screenshot',str(a.output/'gourd-and-oni.png'))
  console(f'item replace entity {name} weapon.offhand from entity {name} weapon.mainhand');console(f'item replace entity {name} weapon.mainhand with minecraft:air');time.sleep(.3);ctl(d,'mouse','right','down');time.sleep(.5);assert ctl(d,'status')['using'] is None;ctl(d,'mouse','right','up');evidence['cooldown_shared_between_hands']=True
  print('PASS: starting kit, drink completion/cancellation, icon and retained gourd; shared-hand cooldown',flush=True)
  # Compare Oni's unarmed hit with vanilla Strength II damage and Knockback II.
  def mob():return next((m for m in ctl(d,'mobs','32')['near'] if m['kind']=='minecraft:pig'),None)
  def hit():
   console('kill @e[type=minecraft:pig]');time.sleep(.3);console('summon minecraft:pig 0.5 -60 2.5 {Silent:1b,PersistenceRequired:1b,attributes:[{id:"minecraft:movement_speed",base:0.0}]}');m=until(mob,lambda m:m is not None and m['health']==10,seconds=6)
   console(f'tp {name} 0.5 -60 0.5 0 0');time.sleep(1);ctl(d,'attack',str(m['id']));m=until(mob,lambda m:m is not None and m['health']<10,seconds=5);health=m['health'];time.sleep(.7);m=mob();return health,m['pos'][2]-2.5
  console(f'item replace entity {name} weapon.offhand with minecraft:air');console(f'effect clear {name}');console(f'item replace entity {name} weapon.mainhand with minecraft:diamond_sword[minecraft:enchantments={{"minecraft:knockback":2}}]');time.sleep(.4);assert 'knockback' in json.dumps(ctl(d,'inventory'));reference=hit();assert abs(reference[0]-3)<.01 and reference[1]>.5
  console(f'item replace entity {name} weapon.mainhand with minecraft:air');console(f'effect give {name} lunatic:oni 15 0 true');until(lambda:oni(d),lambda e:e is not None,seconds=4);boosted=hit();assert abs(boosted[0]-3)<.01;assert abs(boosted[1]-reference[1])<.6,(reference,boosted);evidence['oni_matches_strength_ii_and_knockback_ii']={'vanilla_sword_health_displacement':reference,'oni_unarmed_health_displacement':boosted}
  before_clear_burps=ctl(d,'status')['heard'].get('minecraft:entity.player.burp',0);before_clear_food=player(d)['food']
  console(f'effect clear {name} lunatic:oni');until(lambda:oni(d),lambda e:e is None,seconds=4);time.sleep(.3);assert player(d)['food']==before_clear_food;assert ctl(d,'status')['heard'].get('minecraft:entity.player.burp',0)==before_clear_burps;evidence['early_removal_has_no_expiry_penalty']=True;plain=hit();assert abs(plain[0]-9)<.01 and plain[1]<boosted[1]*.7;evidence['removal_restores_normal_damage_and_knockback']=True
  print('PASS: Oni melee damage/knockback matches vanilla reference; removal restores normal attacks',flush=True)
  # Advance the isolated server's standard ticks, never wall-clock production time.
  console('tick sprint 300');time.sleep(2);console(f'item replace entity {name} weapon.mainhand with lunatic:ibuki_gourd');time.sleep(.3);ctl(d,'mouse','right','down');time.sleep(.5);assert ctl(d,'status')['using'] is None;ctl(d,'mouse','right','up');evidence['cooldown_survives_effect_removal_and_new_copy']=True
  console('tick sprint 4800');time.sleep(2)
  ctl(d,'mouse','right','down');until(lambda:oni(d),lambda e:e is not None,seconds=10);ctl(d,'mouse','right','up');evidence['usable_again_after_four_minute_cooldown']=True
  console(f'effect give {name} minecraft:saturation 1 0 true');time.sleep(.5);console(f'effect clear {name} minecraft:saturation');until(lambda:player(d)['food'],lambda food:food==20,seconds=4)
  burps=ctl(d,'status')['heard'].get('minecraft:entity.player.burp',0)
  until(lambda:oni(d),lambda e:e is None,seconds=23);evidence['oni_expires_naturally']=True
  after=until(lambda:player(d)['food'],lambda food:15<=food<=18,seconds=4);evidence['expiry_food_loss']=20-after
  until(lambda:ctl(d,'status')['heard'].get('minecraft:entity.player.burp',0),lambda n:n==burps+1,seconds=4);time.sleep(.3);assert ctl(d,'status')['heard'].get('minecraft:entity.player.burp',0)==burps+1;evidence['one_normal_burp_on_expiry']=True
  drunk=until(lambda:next((e for e in player(d)['effects'] if e['id']=='lunatic:drunk'),None),lambda e:e is not None,seconds=4);assert 110<=drunk['duration_ticks']<=200 and drunk['amplifier']==0;evidence['drunk_for_random_six_to_ten_seconds']=True
  assert not any(e['id'] in ['darkness','nausea'] for e in player(d)['effects']);evidence['drunk_is_one_synchronized_effect']=True
  time.sleep(2);ctl(d,'screenshot',str(a.output/'drunk-berries.png'));until(lambda:any(e['id']=='lunatic:drunk' for e in player(d)['effects']),lambda active:not active,seconds=12);evidence['drunk_expires_naturally']=True
  heard=ctl(d,'status')['heard'];deaths=heard.get('minecraft:entity.player.death',0);burps=heard.get('minecraft:entity.player.burp',0);console(f'effect give {name} lunatic:oni 1 0 true');console(f'kill {name}');until(lambda:ctl(d,'status')['heard'].get('minecraft:entity.player.death',0),lambda n:n==deaths+1,seconds=5);time.sleep(.4);assert ctl(d,'status')['heard'].get('minecraft:entity.player.burp',0)==burps;evidence['death_sound_once_without_expiry_burp']=True
  (a.output/'result.json').write_text(json.dumps(evidence,indent=2)+'\n');print(json.dumps({'passed':True,'checks':evidence}),flush=True)
 finally:
  for child in reversed(children):
   if child.poll() is None:
    try:child.stdin.write(b'stop\n');child.stdin.flush();child.wait(timeout=3)
    except (BrokenPipeError,OSError,subprocess.TimeoutExpired):child.terminate()
  for child in children:
   try:child.wait(timeout=10)
   except subprocess.TimeoutExpired:child.kill();child.wait()
