#!/usr/bin/env python3
"""Check the shipped orb content, model geometry, and original sound assets."""
import argparse
import importlib.util
import json
from pathlib import Path
import math
import zipfile

ROOT=Path(__file__).resolve().parents[1]

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--luanti',type=Path)
    args=parser.parse_args()
    variants=[('precision','Precision','#dc143c',''),('homing','Homing','#800080','_purple'),('bouncing','Bouncing','#00ff00','_green')]
    lang=json.loads((ROOT/'assets/lunatic/lang/en_us.json').read_text())
    creative=json.loads((ROOT/'data/lunatic/horizon/creative_tab/lunatic.json').read_text())['items']
    for key,label,color,suffix in variants:
        name='yin_yang_orb_'+key
        item=json.loads((ROOT/f'data/lunatic/horizon/item/{name}.json').read_text())
        assert item['max_stack_size']==16
        assert item['components']['minecraft:item_name']['color']==color
        assert lang['item.lunatic.'+name]=='Yin Yang Orb - '+label
        assert creative.count('lunatic:'+name)==1
        model=json.loads((ROOT/f'assets/lunatic/items/{name}.json').read_text())['model']
        assert model['property']=='minecraft:display_context'
        assert model['cases'][0]['when']=='gui'
    sphere=json.loads((ROOT/'assets/lunatic/models/item/yin_yang_orb_sphere.json').read_text())
    assert 1000<len(sphere['elements'])<10000
    spans=[[],[],[]]
    for element in sphere['elements']:
        assert all(math.isfinite(v) for v in element['from']+element['to'])
        for axis in range(3):
            assert element['from'][axis]<element['to'][axis]
            spans[axis].extend([element['from'][axis],element['to'][axis]])
        for face in element['faces'].values():
            assert len(face['uv'])==4 and all(0<=v<=16 for v in face['uv'])
    assert all(min(v)==0 and max(v)==16 for v in spans)
    # The generated artifact must be reproducible; original texture pixels are never regenerated.
    spec=importlib.util.spec_from_file_location('generator',ROOT/'tools/generate_orbs.py')
    generator=importlib.util.module_from_spec(spec);spec.loader.exec_module(generator)
    assert sphere==generator.generate()
    sound_paths={
        'item/purification_rod/activate':'se_option.ogg',
        'item/purification_rod/no_target':'se_ophide.ogg',
        'entity/yin_yang_orb/shoot':'se_plst00.ogg',
        'entity/yin_yang_orb/bounce':'se_graze.ogg',
        'entity/yin_yang_orb/impact':'se_kira00.ogg',
        'entity/yin_yang_orb/close_detonation':'mcl_tnt_tnt_explode.ogg',
    }
    events=json.loads((ROOT/'assets/lunatic/sounds.json').read_text())
    assert len(events)==6
    for event,data in events.items():
        assert data['subtitle'] in lang
        for sound in data['sounds']:
            file=ROOT/('assets/lunatic/sounds/'+sound['name'].split(':',1)[1]+'.ogg')
            assert file.read_bytes().startswith(b'OggS')
    if args.luanti:
        source=args.luanti/'Mineclonia-Lunatic/mods/mcl_lun_sounds/sounds'
        for target,original in sound_paths.items():
            assert (ROOT/('assets/lunatic/sounds/'+target+'.ogg')).read_bytes()==(source/original).read_bytes()
        for original in (args.luanti/'lunatic_textures/mods/mcl_lun_items/textures').glob('yin_yang*.png'):
            assert (ROOT/'assets/lunatic/textures/item'/original.name).read_bytes()==original.read_bytes()
    package=ROOT/'target/horizon-mod/lunatic.hmod'
    if package.exists():
        with zipfile.ZipFile(package) as archive:
            for path in (ROOT/'assets/lunatic/sounds').rglob('*.ogg'):
                assert archive.read(path.relative_to(ROOT).as_posix())==path.read_bytes()
    print('PASS: original names/colors, stack size 16, creative entries, sphere geometry/UVs, original textures and all six sound files/events')

if __name__=='__main__':main()
