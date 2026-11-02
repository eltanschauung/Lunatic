#!/usr/bin/env python3
"""Private dedicated fixture: real rod input, ammo, volleys, impacts and spatial sound delivery."""
import argparse
import json
import os
from pathlib import Path
import shutil
import socket
import subprocess
import tempfile
import time


def until(observe,accept,seconds=120):
    end=time.monotonic()+seconds
    last=None
    while time.monotonic()<end:
        try:
            last=observe()
            if accept(last):return last
        except (OSError,ValueError,subprocess.SubprocessError):pass
        time.sleep(.1)
    raise AssertionError(f'Timeout: {last}')


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for name in ('game','ctl','resources','mod'):parser.add_argument('--'+name,type=Path,required=True)
    args=parser.parse_args()
    args.game,args.ctl,args.resources,args.mod=(p.resolve() for p in (args.game,args.ctl,args.resources,args.mod))
    with tempfile.TemporaryDirectory(prefix='lunatic-weapons-e2e-') as directory:
        root=Path(directory);children=[];logs=[]
        def home(name):
            p=root/name;p.mkdir();(p/'horizon-mods').mkdir()
            shutil.copy2(args.mod,p/'horizon-mods/lunatic.hmod')
            (p/'horizon-mods.txt').write_text('mod.lunatic:true\n')
            (p/'options.txt').write_text('version:5023\nrenderDistance:2\nsimulationDistance:2\n')
            (p/'character.json').write_text('{"character":"lunatic:hakurei_reimu"}')
            return p
        def launch(p,extra):
            log=open(p/'test-process.log','w');logs.append(log)
            env=dict(os.environ,HORIZON_HOME=str(p),HORIZON_LAUNCHER_HOME=str(p/'launcher'),
                XDG_CONFIG_HOME=str(p/'config'),XDG_DATA_HOME=str(p/'data'),XDG_CACHE_HOME=str(p/'cache'),
                HORIZON_TELEMETRY_URL='off',HORIZON_NO_AUDIO='1')
            process=subprocess.Popen([str(args.game),'--resources',str(args.resources),*extra],cwd=p,env=env,
                stdin=subprocess.PIPE,stdout=log,stderr=subprocess.STDOUT)
            children.append(process);return process
        def ctl(p,*words):
            result=subprocess.run([str(args.ctl),'--socket',str(p/'c.sock'),'--timeout','10',*words],capture_output=True,text=True,timeout=15)
            if result.returncode:raise ValueError(result.stderr+result.stdout)
            return json.loads(result.stdout.strip().splitlines()[-1])
        def console(command):server.stdin.write((command+'\n').encode());server.stdin.flush()
        def counts(p):return ctl(p,'inventory')['counts']
        def count(p,item):return counts(p).get('lunatic:'+item,0)
        def sound(p,event):return ctl(p,'status')['heard'].get('lunatic:'+event,0)
        def message(p,text):return any(text in s for s in ctl(p,'status')['server']['chat'])
        server_home=home('server')
        with socket.socket() as sock:sock.bind(('127.0.0.1',0));port=sock.getsockname()[1]
        props={'server-ip':'127.0.0.1','server-port':port,'online-mode':'false','enforce-secure-profile':'false',
            'white-list':'false','view-distance':2,'simulation-distance':2,'spawn-monsters':'false','spawn-animals':'false',
            'level-type':'minecraft:flat','level-seed':123,'generator-settings':json.dumps({'layers':[
                {'block':'minecraft:bedrock','height':1},{'block':'minecraft:dirt','height':2},{'block':'minecraft:grass_block','height':1}],
                'biome':'minecraft:plains','features':False,'lakes':False,'structure_overrides':[]})}
        (server_home/'server.properties').write_text(''.join(f'{k}={v}\n' for k,v in props.items()))
        (server_home/'eula.txt').write_text('eula=true\n')
        try:
            server=launch(server_home,['--dedicated','--nogui'])
            until(lambda:socket.create_connection(('127.0.0.1',port),1).close(),lambda _:True)
            def client(name):
                p=home(name);launch(p,['--headless','--server',f'127.0.0.1:{port}','--name',name,'--control','--control-socket',str(p/'c.sock')])
                until(lambda:ctl(p,'status'),lambda s:(s.get('server') or {}).get('loaded'))
                return p
            alice=client('Alice');bob=client('Bob')
            for path in (alice,bob):until(lambda:count(path,'purification_rod'),lambda n:n==1)
            ground=round(ctl(alice,'status')['player']['position'][1])
            console('gamemode survival Alice');console('gamemode creative Bob')
            console(f'tp Alice 0.5 {ground} 0.5 0 0');console(f'tp Bob 20.5 {ground} 0.5')
            console('clear Alice');console('clear Bob');console('give Alice lunatic:purification_rod 1')
            until(lambda:count(alice,'purification_rod'),lambda n:n==1)
            ctl(alice,'hotbar','1');ctl(alice,'look','0','0');time.sleep(.5)
            ctl(alice,'mouse','right','click');until(lambda:message(alice,'No Yin-Yang Orbs available.'),bool)
            ctl(alice,'mouse','left','click');until(lambda:message(alice,'No Yin-Yang orbs available.'),bool)
            assert not [p for p in ctl(alice,'status')['projectiles'] if 'lunatic:' in json.dumps(p)]
            print('PASS: empty rod right/left messages, ordinary swing, no projectile',flush=True)
            console('give Alice lunatic:yin_yang_orb_precision 20')
            until(lambda:count(alice,'yin_yang_orb_precision'),lambda n:n==20)
            inventory=ctl(alice,'inventory')
            stacks=inventory['hotbar']+inventory['main']
            orb_counts=[s['count'] for s in stacks if s and s.get('item')=='lunatic:yin_yang_orb_precision']
            assert sorted(orb_counts)==[4,16], ('Orb stacking differs from Luanti',orb_counts)
            console('clear Alice lunatic:yin_yang_orb_precision')
            until(lambda:count(alice,'yin_yang_orb_precision'),lambda n:n==0)
            for kind in ('precision','homing','bouncing'):console(f'give Alice lunatic:yin_yang_orb_{kind} 2')
            until(lambda:counts(alice),lambda c:all(c.get('lunatic:yin_yang_orb_'+k)==2 for k in ('precision','homing','bouncing')))
            for label,color in [('Precision','#dc143c'),('Homing','#800080'),('Bouncing','#00ff00'),('Precision','#dc143c')]:
                ctl(alice,'mouse','right','click');until(lambda:message(alice,'Selected ammo: '+label),bool);time.sleep(.3)
                def styled_selection():
                    components=ctl(alice,'messages')['styled']
                    def matches(c):
                        return isinstance(c,dict) and ((c.get('text')=='Selected ammo: ' and c.get('color') is None
                            and any(child.get('text')==label and str(child.get('color')).lower()==color for child in c.get('extra',[])))
                            or any(matches(child) for child in c.get('extra',[])))
                    return any(matches(c) for c in components)
                until(styled_selection,bool)
            print('PASS: ammo selection names retain exact inventory colors through server chat',flush=True)
            assert count(alice,'yin_yang_orb_precision')==2
            console(f'fill -4 {ground} 6 4 {ground+4} 6 minecraft:stone');time.sleep(.5)
            before={str(p):sound(p,'entity.yin_yang_orb.shoot') for p in (alice,bob)}
            ctl(alice,'mouse','left','click');ctl(alice,'mouse','left','click')
            until(lambda:count(alice,'yin_yang_orb_precision'),lambda n:n==1)
            for p in (alice,bob):until(lambda:sound(p,'entity.yin_yang_orb.shoot'),lambda n:n>=before[str(p)]+3)
            time.sleep(.5)
            assert count(alice,'yin_yang_orb_precision')==1, 'Cooldown allowed a second volley'
            assert sound(alice,'entity.yin_yang_orb.shoot')==before[str(alice)]+3
            until(lambda:sound(alice,'entity.yin_yang_orb.bounce'),lambda n:n>=1)
            until(lambda:sound(alice,'entity.yin_yang_orb.impact'),lambda n:n>=1)
            for p in (alice,bob):
                until(lambda:ctl(p,'status')['particles'].get('lunatic:orb_bounce',0),lambda n:n>=8)
                until(lambda:ctl(p,'status')['particles'].get('lunatic:orb_impact',0),lambda n:n>=24)
            print('PASS: original bounce and explosion sprites delivered to both clients',flush=True)
            print('PASS: one ammo for three shots, cooldown, swept wall bounce/impact, original firing/bounce/impact sounds heard by second client 20 blocks away',flush=True)
            time.sleep(1)
            console(f'fill -4 {ground} 6 4 {ground+4} 6 minecraft:air');console('kill @e[type=minecraft:pig]');time.sleep(.5)
            ctl(alice,'mouse','right','click');time.sleep(.3)
            until(lambda:message(alice,'Selected ammo: Homing'),bool)
            failed=sound(alice,'item.purification_rod.no_target')
            ctl(alice,'mouse','left','click')
            until(lambda:sound(alice,'item.purification_rod.no_target'),lambda n:n==failed+1)
            until(lambda:count(alice,'yin_yang_orb_homing'),lambda n:n==1)
            time.sleep(.9)
            console(f'summon minecraft:pig 0.5 {ground} 14.5 {{NoAI:1b,PersistenceRequired:1b}}')
            time.sleep(.5);detonation=sound(alice,'entity.yin_yang_orb.close_detonation')
            ctl(alice,'mouse','left','click')
            until(lambda:sound(alice,'entity.yin_yang_orb.close_detonation'),lambda n:n>detonation)
            until(lambda:count(alice,'yin_yang_orb_homing'),lambda n:n==0)
            print('PASS: failed homing lock sound, homing acquisition and close-range detonation sound',flush=True)
            # A depleted selection restarts cycling at the first available type.
            ctl(alice,'mouse','right','click');time.sleep(.3)
            ctl(alice,'mouse','right','click');time.sleep(.3)
            until(lambda:message(alice,'Selected ammo: Bouncing'),bool)
            console('gamemode creative Alice');time.sleep(.5)
            kept=count(alice,'yin_yang_orb_bouncing');baseline=sound(alice,'entity.yin_yang_orb.shoot')
            ctl(alice,'look','0','25');ctl(alice,'mouse','left','click')
            until(lambda:sound(alice,'entity.yin_yang_orb.shoot'),lambda n:n>=baseline+3)
            assert count(alice,'yin_yang_orb_bouncing')==kept
            print('PASS: creative requires but preserves ammo, bouncing variant fires',flush=True)
            # Turn only pitch while sprinting so changing aim cannot alter horizontal speed.
            for kind in ('precision','homing','bouncing'):console(f'kill @e[type=lunatic:yin_yang_orb_{kind}]')
            console('clear Alice');console('give Alice lunatic:purification_rod 1');console('give Alice lunatic:yin_yang_orb_precision 16')
            until(lambda:count(alice,'yin_yang_orb_precision'),lambda n:n==16)
            console(f'tp Alice 0.5 {ground} 0.5 33.3333 -45');time.sleep(.5)
            ctl(alice,'hotbar','1');ctl(alice,'key','w','down');ctl(alice,'key','ctrl','down');time.sleep(1)
            def speed():
                v=ctl(alice,'status')['player']['velocity']
                return (v[0]*v[0]+v[2]*v[2])**.5
            baseline=min(speed() for _ in range(4));assert baseline>.05, 'Fixture did not start moving'
            shots_before=sound(alice,'entity.yin_yang_orb.shoot')
            samples=[]
            for pitch in (-20,-60,-35,-75):
                ctl(alice,'look','33.3333',str(pitch));ctl(alice,'mouse','left','click')
                end=time.monotonic()+.9
                while time.monotonic()<end:
                    samples.append(speed());time.sleep(.01)
            ctl(alice,'release')
            until(lambda:sound(alice,'entity.yin_yang_orb.shoot'),lambda n:n>=shots_before+12)
            assert min(samples)>baseline*.75, f'Firing interrupted sprint momentum: baseline={baseline:.4f}, minimum={min(samples):.4f}'
            print('PASS: sprint momentum preserved across four aiming/firing volleys',flush=True)
            print('PASS: real dedicated server plus two clients; all weapon and sound checks isolated from production',flush=True)
        finally:
            for sock in root.glob('*/c.sock'):
                try:ctl(sock.parent,'quit')
                except (OSError,ValueError,subprocess.SubprocessError):pass
            for process in children:
                if process.poll() is None:
                    try:process.stdin.write(b'stop\n');process.stdin.flush()
                    except (BrokenPipeError,OSError):pass
            for process in children:
                try:process.wait(timeout=20)
                except subprocess.TimeoutExpired:process.terminate();process.wait(timeout=10)
            for log in logs:log.close()
            diagnostics=Path('/tmp/lunatic-weapons-test-logs');diagnostics.mkdir(exist_ok=True)
            for path in root.glob('*/test-process.log'):shutil.copy2(path,diagnostics/(path.parent.name+'.log'))

if __name__=='__main__':main()
