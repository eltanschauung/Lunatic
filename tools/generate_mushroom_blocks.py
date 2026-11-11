#!/usr/bin/env python3
"""Calibrated multiply-tint vanilla mushroom caps using each terracotta's mean sRGB color.

Vanilla inputs and derived PNGs remain local; only this generator and authored
block definitions belong in Git. No image-generation model is used here.
"""
import argparse
import json
import math
import os
from pathlib import Path
import struct
import zlib

ROOT = Path(__file__).resolve().parents[1]
# Calibrated to selected preview A: mean light-blue output RGB (72,52,72).
# This corrects the warm source texture's blue-channel bias.
TINT_GAIN = (1.088003, 1.088223, 1.633296)
COLORS = ('plain', 'white', 'light_gray', 'gray', 'black', 'brown', 'red',
          'blue', 'light_blue', 'purple', 'magenta')


def read_png(path):
    data = path.read_bytes()
    if data[:8] != b'\x89PNG\r\n\x1a\n':
        raise ValueError(f'Not PNG: {path}')
    width, height, bits, kind, compression, filtering, interlace = struct.unpack('>IIBBBBB', data[16:29])
    if interlace or compression or filtering or kind not in (2, 3, 6):
        raise ValueError(f'Unsupported PNG encoding: {path}')
    palette, alpha, packed, cursor = [], b'', b'', 8
    while cursor < len(data):
        length = int.from_bytes(data[cursor:cursor+4], 'big')
        tag, payload = data[cursor+4:cursor+8], data[cursor+8:cursor+8+length]
        cursor += length + 12
        if tag == b'PLTE': palette = [tuple(payload[i:i+3]) for i in range(0,len(payload),3)]
        if tag == b'tRNS': alpha = payload
        if tag == b'IDAT': packed += payload
    channels = 1 if kind == 3 else 3 if kind == 2 else 4
    if (kind != 3 and bits != 8) or (kind == 3 and bits not in (1,2,4,8)):
        raise ValueError(f'Unsupported PNG depth: {path}')
    stride, bpp = math.ceil(width * bits * channels / 8), max(1, math.ceil(bits*channels/8))
    raw, previous, pixels = zlib.decompress(packed), [0]*stride, []
    for y in range(height):
        mode = raw[y*(stride+1)]
        row = list(raw[y*(stride+1)+1:(y+1)*(stride+1)])
        for x in range(stride):
            a, b, c = row[x-bpp] if x>=bpp else 0, previous[x], previous[x-bpp] if x>=bpp else 0
            p = a+b-c
            distances = [abs(p-a),abs(p-b),abs(p-c)]
            paeth = (a,b,c)[distances.index(min(distances))]
            row[x] = (row[x] + (0,a,b,(a+b)//2,paeth)[mode]) & 255
        for x in range(width):
            if kind == 3:
                index = row[x*bits//8] >> (8-bits-x*bits%8) & ((1<<bits)-1)
                pixels.append((*palette[index], alpha[index] if index<len(alpha) else 255))
            else:
                pixel = tuple(row[x*channels:(x+1)*channels])
                pixels.append((*pixel,255) if kind == 2 else pixel)
        previous = row
    return width, height, pixels


def png_bytes(width, height, pixels):
    def chunk(kind, data):
        return struct.pack('>I',len(data))+kind+data+struct.pack('>I',zlib.crc32(kind+data)&0xffffffff)
    raw = b''.join(b'\0'+bytes(v for pixel in pixels[y*width:(y+1)*width] for v in pixel) for y in range(height))
    return (b'\x89PNG\r\n\x1a\n'+chunk(b'IHDR',struct.pack('>IIBBBBB',width,height,8,6,0,0,0))
            +chunk(b'IDAT',zlib.compress(raw,9))+chunk(b'IEND',b''))


def find_assets(explicit=None):
    if explicit: candidates = [Path(explicit)]
    elif os.environ.get('HORIZON_MINECRAFT_ASSETS'): candidates = [Path(os.environ['HORIZON_MINECRAFT_ASSETS'])]
    else:
        home = Path(os.environ.get('HORIZON_LAUNCHER_HOME',str(Path.home()/'.local/share/horizon-launcher')))
        candidates = sorted(home.glob('fork-versions/*/.local/horizon-client/26.3/assets/minecraft'),
                            key=lambda p:p.stat().st_mtime, reverse=True)
    for path in candidates:
        if (path/'textures/block/brown_mushroom_block.png').is_file(): return path
    raise SystemExit('Prepared vanilla assets missing; pass --minecraft-assets PATH or HORIZON_MINECRAFT_ASSETS')


def write_json(path, value):
    path.parent.mkdir(parents=True,exist_ok=True)
    path.write_text(json.dumps(value,indent=2)+'\n')


def definitions():
    language_path = ROOT/'assets/lunatic/lang/en_us.json'
    language = json.loads(language_path.read_text())
    tab_path = ROOT/'data/lunatic/horizon/creative_tab/lunatic.json'
    tab = json.loads(tab_path.read_text())
    axes = {'north':{},'east':{'y':90},'south':{'y':180},'west':{'y':270},'up':{'x':270},'down':{'x':90}}
    for color in COLORS:
        name = color+'_mushroom_block'; identifier = 'lunatic:'+name
        write_json(ROOT/f'data/lunatic/horizon/block/{name}.json', {
            'template':'minecraft:brown_mushroom_block','item':True,
            'components':{'minecraft:item_name':{'translate':'block.lunatic.'+name}}})
        parts = []
        for face,rotation in axes.items():
            for outer in (True,False):
                model = 'lunatic:block/'+name if outer else 'minecraft:block/mushroom_block_inside'
                parts.append({'when':{face:str(outer).lower()},'apply':{'model':model,**rotation,**({'uvlock':True} if outer and rotation else {})}})
        write_json(ROOT/f'assets/lunatic/blockstates/{name}.json',{'multipart':parts})
        texture = 'lunatic:block/'+name
        write_json(ROOT/f'assets/lunatic/models/block/{name}.json',{
            'parent':'minecraft:block/template_single_face','textures':{'texture':texture,'particle':texture}})
        write_json(ROOT/f'assets/lunatic/models/item/{name}.json',{
            'parent':'minecraft:block/cube_all','textures':{'all':texture}})
        write_json(ROOT/f'assets/lunatic/items/{name}.json',{'model':{'type':'minecraft:model','model':'lunatic:item/'+name}})
        write_json(ROOT/f'data/lunatic/loot_table/blocks/{name}.json',{'type':'minecraft:block','pools':[{
            'rolls':1,'entries':[{'type':'minecraft:item','name':identifier}],
            'conditions':[{'condition':'minecraft:survives_explosion'}]}]})
        language['block.lunatic.'+name] = color.replace('_',' ').title()+' Mushroom Block'
        if identifier not in tab['items']: tab['items'].append(identifier)
    write_json(language_path,language)
    write_json(tab_path,tab)
    axe_path = ROOT/'data/minecraft/tags/block/mineable/axe.json'
    axe = json.loads(axe_path.read_text()) if axe_path.exists() else {'replace':False,'values':[]}
    existing = {v if isinstance(v,str) else v['id'] for v in axe['values']}
    axe['values'].extend('lunatic:'+c+'_mushroom_block' for c in COLORS if 'lunatic:'+c+'_mushroom_block' not in existing)
    write_json(axe_path,axe)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--minecraft-assets',type=Path)
    parser.add_argument('--definitions',action='store_true',help='Also author block/item definitions and creative entries')
    args = parser.parse_args()
    assets = find_assets(args.minecraft_assets)
    width,height,original = read_png(assets/'textures/block/brown_mushroom_block.png')
    assert (width,height)==(16,16)
    report = {}
    for color in COLORS:
        source = 'terracotta' if color=='plain' else color+'_terracotta'
        _,_,samples = read_png(assets/f'textures/block/{source}.png')
        mean = [round(sum(p[i] for p in samples)/len(samples)) for i in range(3)]
        tinted = [tuple(min(255, int(p[i]*mean[i]*TINT_GAIN[i]/255 + 0.5)) for i in range(3))+(p[3],) for p in original]
        name = color+'_mushroom_block'
        target = ROOT/f'assets/lunatic/textures/block/{name}.png'
        target.parent.mkdir(parents=True,exist_ok=True)
        target.write_bytes(png_bytes(width,height,tinted))
        assert read_png(target)==(width,height,tinted)
        report[name] = {'source':source,'target_rgb':mean,'target_hex':'#'+''.join(f'{v:02x}' for v in mean),'tint_gain':TINT_GAIN}
    output=ROOT/'.local/mushroom-texture-previews';output.mkdir(parents=True,exist_ok=True)
    write_json(output/'generated-colors.json',report)
    if args.definitions: definitions()
    print(f'Generated {len(COLORS)} exact 16x16 calibrated multiply-tinted caps; vanilla interiors unchanged')

if __name__=='__main__': main()
