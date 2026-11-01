#!/usr/bin/env python3
"""Generate the exposed shell of a 32³ voxel sphere with longitude/latitude UVs repeated on both vertical hemispheres."""
import json
import math
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SIZE = 32
DIRECTIONS = {'west':(-1,0,0),'east':(1,0,0),'down':(0,-1,0),
              'up':(0,1,0),'north':(0,0,-1),'south':(0,0,1)}

def occupied(x,y,z):
    return sum((n+0.5-SIZE/2)**2 for n in (x,y,z)) <= (SIZE/2)**2

def uv(point, center):
    x,y,z = (v-8 for v in point)
    length = math.sqrt(x*x+y*y+z*z)
    # Each front/back hemisphere spans one complete copy. Keep the seam at
    # z=0 on its own side so a voxel face never stretches across the repeat.
    u = 0.5 + math.atan2(x,z)/math.pi if x or z else center
    while u-center > 1: u-=2
    while u-center < -1: u+=2
    u -= math.floor(center)
    return max(0,min(1,u))*16, math.acos(max(-1,min(1,y/length)))/math.pi*16

def generate():
    elements=[]
    for x in range(SIZE):
        for y in range(SIZE):
            for z in range(SIZE):
                if not occupied(x,y,z):continue
                low=[n/2 for n in (x,y,z)];high=[v+0.5 for v in low]
                faces={}
                for face,d in DIRECTIONS.items():
                    if occupied(x+d[0],y+d[1],z+d[2]):continue
                    axis=next(i for i,v in enumerate(d) if v)
                    a,b=[i for i in range(3) if i!=axis]
                    points=[]
                    for av,bv in ((0,0),(1,0),(1,1),(0,1)):
                        p=list(low);p[axis]=(high if d[axis]>0 else low)[axis]
                        p[a]=(high if av else low)[a];p[b]=(high if bv else low)[b];points.append(p)
                    cx,cz=(low[0]+high[0])/2-8,(low[2]+high[2])/2-8
                    center=0.5+math.atan2(cx,cz)/math.pi
                    coords=[uv(p,center) for p in points]
                    us,vs=zip(*coords)
                    bounds=[min(us),min(vs),max(us),max(vs)]
                    # Orient the rectangular voxel-face UV approximation consistently.
                    if face in ('west','south'):bounds[0],bounds[2]=bounds[2],bounds[0]
                    if face=='down':bounds[1],bounds[3]=bounds[3],bounds[1]
                    faces[face]={'uv':[round(v,6) for v in bounds],'texture':'#orb'}
                if faces:
                    # The source is an RGBA circular sprite. A white inner shell fills
                    # transparent padding without altering any original texture pixels.
                    elements.append({'from':[round(8+(v-8)*0.998,6) for v in low],
                        'to':[round(8+(v-8)*0.998,6) for v in high],
                        'faces':{f:{'uv':[0,0,16,16],'texture':'#core'} for f in faces}})
                    elements.append({'from':low,'to':high,'faces':faces})
    return {'textures':{'core':'minecraft:block/white_concrete','particle':'#orb'},
            'gui_light':'front','display':{
                'firstperson_righthand':{'rotation':[0,-90,25],'translation':[1.13,3.2,1.13],'scale':[0.35]*3},
                'firstperson_lefthand':{'rotation':[0,90,-25],'translation':[1.13,3.2,1.13],'scale':[0.35]*3},
                'thirdperson_righthand':{'rotation':[0,0,0],'translation':[0,3,1],'scale':[0.35]*3},
                'thirdperson_lefthand':{'rotation':[0,0,0],'translation':[0,3,1],'scale':[0.35]*3},
                'ground':{'translation':[0,2,0],'scale':[0.3]*3},
                'fixed':{'rotation':[0,0,0],'translation':[0,0,0],'scale':[1]*3}},
            'elements':elements}

def main():
    p=ROOT/'assets/lunatic/models/item/yin_yang_orb_sphere.json'
    p.write_text(json.dumps(generate(),separators=(',',':'))+'\n')
    print('Generated 32³ voxel sphere:',p.relative_to(ROOT))

if __name__=='__main__':main()
