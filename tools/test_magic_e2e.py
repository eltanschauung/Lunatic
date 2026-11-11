#!/usr/bin/env python3
"""Isolated multiplayer/UI proof for permanent magic and size spells."""
import argparse,json,os,re,shutil,socket,subprocess,tempfile,time,zipfile
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
for key in ('game','server','ctl','resources','packages','output'):p.add_argument('--'+key,type=Path,required=True)
a=p.parse_args()
for key,value in vars(a).items():setattr(a,key,value.resolve())
a.output.mkdir(parents=True,exist_ok=True)
def until(call,predicate=lambda x:bool(x),seconds=12):
 end=time.monotonic()+seconds;last=None
 while time.monotonic()<end:
  try:
   last=call()
   if predicate(last):return last
  except (OSError,ValueError,KeyError,subprocess.SubprocessError):pass
  time.sleep(.08)
 raise AssertionError('Condition timed out: '+str(last)[:500])
with zipfile.ZipFile(a.packages/'inputs/horizon-mods/lunatic.hmod') as pack:
 characters=json.loads(pack.read('assets/lunatic/horizon/characters.json'))['characters']
 normal=float(next(c for c in characters if c['id']=='lunatic:hakurei_reimu').get('size',1))
children=[];evidence={}
if not os.environ.get('DISPLAY') and not os.environ.get('WAYLAND_DISPLAY'):
 display=subprocess.Popen(['Xvfb','-screen','0','800x600x24','-displayfd','1','-nolisten','tcp'],stdout=subprocess.PIPE,stderr=subprocess.DEVNULL,text=True)
 number=display.stdout.readline().strip();assert number.isdigit(),'Xvfb failed to create a test display'
 os.environ['DISPLAY']=':'+number;children.append(display)
with tempfile.TemporaryDirectory(prefix='lunatic-magic-proof-') as temporary:
 root=Path(temporary)
 def home(name):
  d=root/name;d.mkdir();(d/'horizon-mods').mkdir()
  for f in (a.packages/'inputs/horizon-mods').glob('*.hmod'):shutil.copy2(f,d/'horizon-mods'/f.name)
  (d/'horizon-mods.txt').write_text('mod.lunatic:true\n')
  (d/'options.txt').write_text('renderDistance:2\nsimulationDistance:2\nguiScale:2\nmaxFps:45\nautoJump:false\npauseOnLostFocus:false\n')
  return d
 def launch(d,args,label,server=False):
  env=dict(os.environ,HORIZON_HOME=str(d),HORIZON_LAUNCHER_HOME=str(d/'launcher'),XDG_CONFIG_HOME=str(d/'config'),XDG_DATA_HOME=str(d/'data'),XDG_CACHE_HOME=str(d/'cache'),HORIZON_NO_AUDIO='1')
  with (a.output/(label+'.log')).open('w') as log:
   child=subprocess.Popen([str(a.server if server else a.game),'--resources',str(a.resources),'--game-dir',str(d),'--home',str(d),*args],cwd=d,env=env,stdin=subprocess.PIPE,stdout=log,stderr=log)
  children.append(child);return child
 def ctl(d,*args):
  r=subprocess.run([str(a.ctl),'--socket',str(d/'c.sock'),'--timeout','12',*map(str,args)],capture_output=True,text=True,timeout=18)
  if r.returncode:raise ValueError((r.stdout+r.stderr)[-600:])
  return json.loads(r.stdout)
 def status():return ctl(d,'status')
 def body():return status()['player']
 def effects():return ctl(d,'state')['player']['effects']
 def has(effect):return any(e['id']=='lunatic:'+effect for e in effects())
 def console(line):server.stdin.write((line+'\n').encode());server.stdin.flush()
 def connect(d,char,label):
  (d/'character.json').write_text(json.dumps({'character':'lunatic:'+char}))
  child=launch(d,['--offscreen','--server',f'127.0.0.1:{port}','--name','MagicProof','--control','--control-socket',str(d/'c.sock'),'--width','800','--height','600'],label)
  until(lambda:ctl(d,'status'),lambda s:(s.get('server') or {}).get('loaded',False),seconds=90)
  return child
 def open_magic():
  current=status()
  if current.get('screen')!='inventory':ctl(d,'open-inventory')
  s=status()['magic']
  if not s['open']:
   x,y=s['geometry']['button'];ctl(d,'click-at',x+10,y+9)
  return until(lambda:status()['magic'],lambda s:s['open'] and s['ready'])
 def point(index):
  s=status()['magic'];ox,oy=s['geometry']['origin'];i=index%36
  return (ox+8+(i%9)*18+8,oy+(84+(i//9)*18 if i<27 else 142)+8)
 def click_spell(id):
  s=open_magic();slot=s['slots'].index('lunatic:'+id);ctl(d,'click-at',*point(slot))
 def query(attribute):
  previous=set(status()['server']['chat'])
  ctl(d,'command',f'attribute @s minecraft:{attribute} get')
  def read():
   for line in status()['server']['chat']:
    if line not in previous:
     match=re.search(r' is (-?\d+(?:\.\d+)?(?:[Ee][-+]?\d+)?)\s*$',line)
     if match:return float(match.group(1))
   return None
  return until(read,lambda v:v is not None,seconds=4)
 srv=home('server')
 with socket.socket() as s:s.bind(('127.0.0.1',0));port=s.getsockname()[1]
 props={'server-ip':'127.0.0.1','server-port':port,'online-mode':'false','enforce-secure-profile':'false','white-list':'false','view-distance':2,'simulation-distance':2,'spawn-monsters':'false','spawn-animals':'false','level-type':'minecraft:flat','level-seed':123,'generator-settings':json.dumps({'layers':[{'block':'minecraft:bedrock','height':1},{'block':'minecraft:dirt','height':2},{'block':'minecraft:grass_block','height':1}],'biome':'minecraft:plains','features':False,'lakes':False,'structure_overrides':[]})}
 (srv/'server.properties').write_text(''.join(f'{k}={v}\n' for k,v in props.items()));(srv/'eula.txt').write_text('eula=true\n')
 try:
  server=launch(srv,['--dedicated','--nogui'],'server',True)
  until(lambda:socket.create_connection(('127.0.0.1',port),1).close(),lambda _:True,seconds=90)
  console('gamerule spawn_mobs false');console('gamerule spawn_monsters false');console('gamerule natural_regeneration false')
  d=home('client');client=connect(d,'ibuki_suika','client')
  until(lambda:body()['scale'],lambda s:abs(s-.82)<.01)
  log=(a.output/'server.log').read_text();name=re.findall(r'\]: (\S+)\[/127\.0\.0\.1:',log)[0]
  console(f'op {name}');console(f'gamemode survival {name}');console(f'tp {name} 0.5 -60 0.5 0 0');time.sleep(.5)
  console(f'item replace entity {name} armor.head with minecraft:iron_helmet');console(f'item replace entity {name} weapon.offhand with minecraft:shield');console(f'item replace entity {name} hotbar.1 with minecraft:oak_log');time.sleep(.4)
  ctl(d,'open-inventory');ctl(d,'click-slot',1);ctl(d,'click-slot',100)
  ctl(d,'hotbar',9);baseline=body();before=ctl(d,'inventory')['counts']
  s=open_magic();assert s['slots'][:2]==['lunatic:gigantification','lunatic:miniaturization']
  ctl(d,'screenshot',a.output/'magic-menu.png');locked_menu=ctl(d,'menu')
  for slot in (500,504,100,200):ctl(d,'click-slot',slot)
  assert ctl(d,'inventory')['counts']==before
  assert ctl(d,'menu')==locked_menu
  start=point(0);end=point(35);ctl(d,'drag-at',*start,*end)
  s=until(lambda:status()['magic'],lambda s:s['slots'][35]=='lunatic:gigantification');revision=s['revision']
  ctl(d,'drag-at',*end,-20,-20)
  for key in ('q','1','f'):ctl(d,'key',key,'tap')
  assert status()['magic']['revision']==revision and status()['magic']['slots'][35]=='lunatic:gigantification'
  assert ctl(d,'inventory')['counts']==before
  evidence['virtual_page_locks_equipment_crafting_and_dropping']=True;evidence['reorder_preserves_unlocks']=True
  ctl(d,'close');click_spell('gigantification')
  until(lambda:has('growing'))
  assert body()['food']==baseline['food']-6
  growing_speed=query('movement_speed');assert growing_speed<.02,growing_speed
  time.sleep(1);mid=body()['scale'];assert .82<mid<6.56,mid
  until(lambda:has('giant'),seconds=6);until(lambda:body()['scale'],lambda v:abs(v-6.56)<.02)
  assert abs(body()['dimensions'][1]-baseline['dimensions'][1]*8)<.05
  assert abs(query('attack_damage')-7)<.01
  assert status()['heard'].get('minecraft:entity.ender_dragon.growl',0)>0
  ctl(d,'screenshot',a.output/'giant.png');ctl(d,'key','f5','tap');time.sleep(.25);ctl(d,'screenshot',a.output/'giant-thirdperson.png');ctl(d,'key','f5','tap');ctl(d,'key','f5','tap')
  before_explosion=status()['heard'].get('minecraft:entity.generic.explode',0);health=body()['health']
  console('summon minecraft:pig 3 -60 0.5 {Silent:1b,NoAI:1b,PersistenceRequired:1b}');time.sleep(.3)
  ctl(d,'jump')
  until(lambda:status()['heard'].get('minecraft:entity.generic.explode',0),lambda n:n>before_explosion,seconds=5)
  assert body()['health']==health
  assert not any(m['kind']=='minecraft:pig' and m['health']>0 for m in ctl(d,'mobs','16')['near'])
  evidence['giant_growth_cost_strength_and_caster_safe_impact']=True
  until(lambda:has('shrinking'),seconds=23)
  until(lambda:body()['scale'],lambda v:abs(v-.82)<.01,seconds=7)
  assert not has('giant') and not has('shrinking');assert abs(query('attack_damage')-1)<.01
  evidence['giant_restores_baseline']=True
  console('fill -8 -63 -8 8 -61 8 minecraft:dirt');console('fill -8 -60 -8 8 -60 8 minecraft:air');console(f'tp {name} 0.5 -60 0.5 0 0');time.sleep(.5)
  food=body()['food'];click_spell('miniaturization');until(lambda:has('shrinking'));assert body()['food']==food-4
  until(lambda:has('tiny'),seconds=5);until(lambda:body()['scale'],lambda v:abs(v-.164)<.002)
  assert abs(body()['dimensions'][1]-baseline['dimensions'][1]/5)<.02
  assert abs(query('movement_speed')-.15)<.005
  assert abs(query('block_break_speed')-.3)<.005
  assert abs(query('jump_strength')-.62)<.005
  assert abs(query('fall_damage_multiplier'))<.001
  health=body()['health'];console(f'tp {name} 0.5 -36 0.5 0 0')
  until(lambda:body(),lambda b:b['on_ground'] and b['position'][1]<-59,seconds=8)
  assert body()['health']==health
  ctl(d,'screenshot',a.output/'tiny.png');evidence['tiny_cost_scale_speed_mining_jump_and_fall_immunity']=True
  # Rejoining clears transient magic while preserving permanently unlocked layout.
  client.terminate();client.wait(timeout=15);time.sleep(.6)
  client=connect(d,'hakurei_reimu','rejoined');until(lambda:body()['scale'],lambda s:abs(s-normal)<.01,seconds=8)
  s=open_magic();assert s['slots'][35]=='lunatic:gigantification' and s['slots'][1]=='lunatic:miniaturization'
  assert not any(e['id'] in ['lunatic:growing','lunatic:giant','lunatic:shrinking','lunatic:tiny'] for e in effects())
  evidence['rejoin_and_character_change_preserve_layout_and_restore_size']=True
  # Hunger cannot go negative and rejection cannot start a size transition.
  ctl(d,'close')
  # Spend food through real casts, cancelling their transient state by rejoining.
  # This avoids depending on vanilla Hunger-effect timing or administrator selectors.
  for attempt in range(4):
   food=body()['food']
   if food<4:break
   spell='gigantification' if food>=6 else 'miniaturization'
   click_spell(spell);until(lambda:has('growing') or has('shrinking'))
   client.terminate();client.wait(timeout=15);time.sleep(.6)
   client=connect(d,'hakurei_reimu','hungry-'+str(attempt));until(lambda:body()['scale'],lambda s:abs(s-normal)<.01,seconds=8)
  assert body()['food']<4
  click_spell('miniaturization')
  until(lambda:status()['magic'],lambda s:s['error'] is not None)
  assert abs(body()['scale']-normal)<.01 and body()['food']>=0
  evidence['insufficient_hunger_rejected']=True
  (a.output/'evidence.json').write_text(json.dumps(evidence,indent=2)+'\n')
  print(json.dumps({'passed':True,'checks':evidence}),flush=True)
 finally:
  (a.output/'partial-evidence.json').write_text(json.dumps(evidence,indent=2)+'\n')
  for child in reversed(children):
   if child.poll() is None:
    child.terminate()
    try:child.wait(timeout=10)
    except subprocess.TimeoutExpired:child.kill();child.wait()
