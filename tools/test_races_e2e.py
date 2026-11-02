#!/usr/bin/env python3
"""Private multiplayer fixture for Lunatic kits, player dimensions and fairy movement."""
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


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ('game', 'ctl', 'resources', 'mod'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    args.game, args.ctl, args.resources, args.mod = (
        p.resolve() for p in (args.game, args.ctl, args.resources, args.mod))
    with tempfile.TemporaryDirectory(prefix='lunatic-races-e2e-') as scratch:
        root = Path(scratch)
        children, logs = [], []

        def home(name):
            p = root / name
            p.mkdir(exist_ok=True)
            (p / 'horizon-mods').mkdir(exist_ok=True)
            shutil.copy2(args.mod, p / 'horizon-mods/lunatic.hmod')
            (p / 'horizon-mods.txt').write_text('mod.lunatic:true\n')
            (p / 'options.txt').write_text('version:5023\nrenderDistance:2\nsimulationDistance:2\nautoJump:false\n')
            return p

        def launch(p, extra):
            log = open(p / 'fixture.log', 'w')
            logs.append(log)
            env = dict(os.environ, HORIZON_HOME=str(p), HORIZON_LAUNCHER_HOME=str(p/'launcher'),
                       XDG_CONFIG_HOME=str(p/'config'), XDG_DATA_HOME=str(p/'data'),
                       XDG_CACHE_HOME=str(p/'cache'), HORIZON_TELEMETRY_URL='off', HORIZON_NO_AUDIO='1')
            process = subprocess.Popen([str(args.game), '--resources', str(args.resources), *extra],
                                       cwd=p, env=env, stdin=subprocess.PIPE, stdout=log, stderr=subprocess.STDOUT)
            children.append(process)
            return process

        def ctl(p, *words):
            result = subprocess.run([str(args.ctl), '--socket', str(p/'c.sock'), '--timeout', '10', *words],
                                    capture_output=True, text=True, timeout=15)
            if result.returncode:
                raise ValueError(result.stderr + result.stdout)
            return json.loads(result.stdout.strip().splitlines()[-1])

        def counts(p):
            return ctl(p, 'inventory')['counts']

        def player(p):
            return ctl(p, 'state')['player']

        def height(p):
            state = player(p)
            return state['eye'][1] - state['position'][1]

        def console(command):
            server.stdin.write((command + '\n').encode())
            server.stdin.flush()

        def client(name, character):
            p = home(name)
            (p/'character.json').write_text(json.dumps({'character': 'lunatic:' + character}))
            process = launch(p, ['--headless', '--server', f'127.0.0.1:{port}', '--name', name,
                                 '--control', '--control-socket', str(p/'c.sock')])
            until(lambda: ctl(p, 'status'), lambda s: (s.get('server') or {}).get('loaded', False))
            return p, process

        def quit_client(p, process):
            ctl(p, 'quit')
            process.wait(timeout=30)

        def press(p, down=.1, up=.1):
            ctl(p, 'key', 'space', 'down')
            time.sleep(down)
            ctl(p, 'key', 'space', 'up')
            time.sleep(up)

        srv_home = home('server')
        with socket.socket() as sock:
            sock.bind(('127.0.0.1', 0))
            port = sock.getsockname()[1]
        settings = {'server-ip': '127.0.0.1', 'server-port': port, 'online-mode': 'false',
                    'enforce-secure-profile': 'false', 'white-list': 'false', 'view-distance': 2,
                    'simulation-distance': 2, 'spawn-monsters': 'false', 'spawn-animals': 'false',
                    'level-type': 'minecraft:flat', 'level-seed': 123,
                    'generator-settings': json.dumps({'layers': [{'block': 'minecraft:bedrock', 'height': 1},
                         {'block': 'minecraft:dirt', 'height': 2}, {'block': 'minecraft:grass_block', 'height': 1}],
                         'biome': 'minecraft:plains', 'features': False, 'lakes': False, 'structure_overrides': []})}
        (srv_home/'server.properties').write_text(''.join(f'{k}={v}\n' for k, v in settings.items()))
        (srv_home/'eula.txt').write_text('eula=true\n')
        try:
            server = launch(srv_home, ['--dedicated', '--nogui'])
            until(lambda: socket.create_connection(('127.0.0.1', port), 1).close(), lambda _: True)
            alice, a = client('Alice', 'cirno')
            bob, b = client('Bob', 'hakurei_reimu')
            until(lambda: counts(alice), lambda c: c.get('minecraft:cookie') == 16)
            kit = counts(alice)
            flowers = {k: v for k, v in kit.items() if k != 'minecraft:cookie'}
            assert 1 <= len(flowers) <= 3 and 3 <= sum(flowers.values()) <= 12, kit
            until(lambda: counts(bob), lambda c: c.get('lunatic:purification_rod') == 1)
            kit = counts(bob)
            assert all(4 <= kit.get('lunatic:yin_yang_orb_' + v, 0) <= 12
                       for v in ('precision', 'homing', 'bouncing')), kit
            until(lambda: height(alice), lambda h: abs(h - 1.62*.7) < .001)
            assert abs(height(bob) - 1.62) < .001
            print('PASS: catalog kits, bounded random flowers/ammo, fairy and normal eye height', flush=True)
            console('clear Alice')
            until(lambda: counts(alice), lambda c: not c)
            quit_client(alice, a)
            alice, a = client('Alice', 'cirno')
            until(lambda: height(alice), lambda h: abs(h - 1.62*.7) < .001)
            time.sleep(2)
            assert not counts(alice), 'Reconnect duplicated the starting kit'
            print('PASS: consumed kit stays consumed across reconnect', flush=True)

            ground = round(player(alice)['position'][1])
            console('gamemode survival Alice')
            console(f'tp Alice 0.5 {ground} 0.5 0 0')
            until(lambda: player(alice), lambda p: p['on_ground'])
            def jumps():
                return ctl(alice, 'status')['heard'].get('lunatic:entity.fairy.jump', 0)
            first = jumps()
            press(alice, .10, .05)
            assert jumps() == first, 'Normal ground jump consumed the airborne boost'
            press(alice, .05, .05)
            until(jumps, lambda n: n == first + 1)
            assert player(alice)['position'][1] > ground + .5
            until(lambda: player(alice)['velocity'][1], lambda v: v < 0)
            falling = player(alice)['velocity'][1]
            assert -.5 < falling < 0, falling
            # Repeated presses cancel float, but never grant a second boost.
            press(alice, .05, .05)
            after = player(alice)['velocity'][1]
            assert after < .1, after
            console(f'tp Alice 0.5 {ground} 0.5')
            until(lambda: player(alice), lambda p: p['on_ground'])
            print('PASS: airborne boost, slow falling and cancellation without another boost', flush=True)

            # Start slow fall again, cancel its race source while a potion is active.
            console(f'tp Alice 0.5 {ground+20} 0.5')
            time.sleep(.25)
            press(alice, .05, .1)
            console('effect give Alice minecraft:slow_falling 30 0 true')
            until(lambda: player(alice), lambda p: any(e['id']=='slow_falling' for e in p['effects']))
            press(alice, .05, .05)
            console(f'tp Alice 0.5 {ground+20} 0.5')
            time.sleep(1)
            assert -.5 < player(alice)['velocity'][1] <= 0, player(alice)
            console('effect clear Alice minecraft:slow_falling')
            time.sleep(.8)
            assert player(alice)['velocity'][1] < -.5, player(alice)
            console(f'tp Alice 0.5 {ground} 0.5')
            print('PASS: cancelling fairy float preserves potion Slow Falling; potion expiry restores gravity', flush=True)

            # Re-selecting an ordinary character removes the fairy trait and size.
            quit_client(alice, a)
            alice, a = client('Alice', 'kirisame_marisa')
            until(lambda: counts(alice), lambda c: c.get('minecraft:experience_bottle') == 16)
            until(lambda: height(alice), lambda h: abs(h - 1.62) < .001)
            quit_client(alice, a)
            alice, a = client('Alice', 'cirno')
            until(lambda: height(alice), lambda h: abs(h - 1.62*.7) < .001)
            assert counts(alice).get('minecraft:cookie', 0) == 0
            print('PASS: selection changes traits and grants each character kit only once', flush=True)
            # A full inventory must keep the one rolled grant across a server restart.
            for group, size in (('hotbar', 9), ('inventory', 27)):
                for slot in range(size):
                    console(f'item replace entity Alice {group}.{slot} with minecraft:stone 64')
            until(lambda: counts(alice), lambda c: c.get('minecraft:stone') == 36*64)
            quit_client(alice, a)
            alice, a = client('Alice', 'kochiya_sanae')
            until(lambda: height(alice), lambda h: abs(h - 1.62) < .001)
            time.sleep(2)
            assert counts(alice).get('minecraft:stone') == 36*64
            assert counts(alice).get('lunatic:miko_stick', 0) == 0
            quit_client(alice, a)
            quit_client(bob, b)
            server.stdin.write(b'stop\n')
            server.stdin.flush()
            server.wait(timeout=60)
            server = launch(srv_home, ['--dedicated', '--nogui'])
            until(lambda: socket.create_connection(('127.0.0.1', port), 1).close(), lambda _: True)
            alice, a = client('Alice', 'kochiya_sanae')
            until(lambda: counts(alice), lambda c: c.get('minecraft:stone') == 36*64)
            console('clear Alice')
            until(lambda: counts(alice), lambda c: c.get('lunatic:miko_stick') == 1)
            quit_client(alice, a)
            alice, a = client('Alice', 'kochiya_sanae')
            time.sleep(2)
            assert counts(alice).get('lunatic:miko_stick') == 1
            print('PASS: full inventory preserves pending kit across server restart; remainder granted once', flush=True)
            quit_client(alice, a)
            server.stdin.write(b'stop\n')
            server.stdin.flush()
            server.wait(timeout=60)
        finally:
            for process in reversed(children):
                if process.poll() is None:
                    process.terminate()
                    try:
                        process.wait(timeout=20)
                    except subprocess.TimeoutExpired:
                        process.kill()
                        process.wait()
            for log in logs:
                log.close()


if __name__ == '__main__':
    main()
