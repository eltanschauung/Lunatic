#!/usr/bin/env python3
"""Reproduce Luanti's ^[colorize:<tint>:180 while preserving the original alpha.

Requires Pillow only when regenerating these checked-in derived assets.
The eight precision sprites are unmodified originals from mcl_lun_items.
"""
from pathlib import Path
import json
from PIL import Image

ROOT = Path(__file__).resolve().parents[1]
sprites = ROOT / 'assets/lunatic/textures/particle/yin_yang_orb'
for i in range(1, 9):
    with Image.open(sprites / f'precision_{i}.png') as source:
        for name, tint in [('homing', (128, 0, 128)), ('bouncing', (0, 255, 0))]:
            image = source.convert('RGBA')
            pixels = image.load()
            image.putdata([tuple((pixels[x, y][c]*75+tint[c]*180)//255 for c in range(3))+(pixels[x, y][3],)
                           for y in range(image.height) for x in range(image.width)])
            image.save(sprites / f'{name}_{i}.png')

# A namespaced Minecraft atlas, with full-image regions. Horizon packs these
# generated atlas entries into the shared sprite atlas used by its particles.
atlas = ROOT / 'assets/lunatic/atlases/orb_particles.json'
atlas.parent.mkdir(parents=True, exist_ok=True)
sources = []
for name in ('precision', 'homing', 'bouncing'):
    for i in range(1, 9):
        sprite = f'lunatic:particle/yin_yang_orb/{name}_{i}'
        sources.append({'type': 'minecraft:unstitch', 'resource': sprite,
                        'divisor_x': 1, 'divisor_y': 1,
                        'regions': [{'sprite': sprite, 'x': 0, 'y': 0, 'width': 1, 'height': 1}]})
atlas.write_text(json.dumps({'sources': sources}, indent=2)+'\n')
print('Generated 16 Luanti particle tints and the 24-sprite atlas.')
