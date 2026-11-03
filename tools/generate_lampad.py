#!/usr/bin/env python3
"""Voxelize Luanti's original star outline at 32 samples across two blocks."""
from pathlib import Path
import json,math
ROOT=Path(__file__).resolve().parents[1]
def generate():
 vertices=[];uv=[];faces=[]
 for line in (ROOT/'tools/reference/lampad_star.obj').read_text().splitlines():
  a=line.split()
  if not a:continue
  if a[0]=='v':vertices.append(tuple(map(float,a[1:])))
  elif a[0]=='vt':uv.append(tuple(map(float,a[1:])))
  elif a[0]=='f':faces.append([tuple(int(n)-1 for n in x.split('/'))for x in a[1:]])
 # The extra centre vertex only triangulates the surface; ten perimeter points define the silhouette.
 outline=sorted([(x,z)for x,y,z in vertices[:20]if y>0.1],key=lambda p:math.atan2(p[1],p[0]))
 def inside(x,z):
  hit=False
  for a,b in zip(outline,outline[1:]+outline[:1]):
   if (a[1]>z)!=(b[1]>z)and x<(b[0]-a[0])*(z-a[1])/(b[1]-a[1])+a[0]:hit=not hit
  return hit
 triangles=[]
 for f in faces:
  if all(vertices[v[0]][1]>0.1 for v in f):
   triangles.extend((f[0],f[i],f[i+1])for i in range(1,len(f)-1))
 def tex(x,z):
  for tri in triangles:
   (a,b,c)=[(vertices[v[0]][0],vertices[v[0]][2])for v in tri]
   det=(b[1]-c[1])*(a[0]-c[0])+(c[0]-b[0])*(a[1]-c[1])
   if abs(det)<1e-12:continue
   u=((b[1]-c[1])*(x-c[0])+(c[0]-b[0])*(z-c[1]))/det
   v=((c[1]-a[1])*(x-c[0])+(a[0]-c[0])*(z-c[1]))/det
   w=1-u-v
   if min(u,v,w)>=-1e-5:
    coords=[uv[p[1]]for p in tri];return [sum(t*q[0]for t,q in zip((u,v,w),coords))*16,(1-sum(t*q[1]for t,q in zip((u,v,w),coords)))*16]
  return [(x+1)*3.2,(z+1)*3.2]
 occupied={(x,y,k)for x in range(32)for y in range(32)for k in range(3)if inside((x+0.5)/16-1,1-(y+0.5)/16)}
 elements=[]
 for x,y,k in sorted(occupied):
  fs={};u,v=tex((x+0.5)/16-1,1-(y+0.5)/16)
  for name,(dx,dy,dz)in {'east':(1,0,0),'west':(-1,0,0),'up':(0,1,0),'down':(0,-1,0),'north':(0,0,-1),'south':(0,0,1)}.items():
   if (x+dx,y+dy,k+dz)in occupied:continue
   bounds=[u-.04,v-.04,u+.04,v+.04]if dz else[7.3+(k/3),1,7.3+((k+1)/3),1.2]
   fs[name]={'texture':'#star','uv':bounds}
  if fs:elements.append({'from':[x-8,y-8,k+6.5],'to':[x-7,y-7,k+7.5],'shade':False,'light_emission':15,'faces':fs})
 model={'ambientocclusion':False,'textures':{'particle':'#star'},'display':{'ground':{'scale':[1.2]*3}},'elements':elements}
 p=ROOT/'assets/lunatic/models/item/lampad_star_voxel.json';p.write_text(json.dumps(model,separators=(',',':'))+'\n')
 for color in ['red','white','blue']:
  p=ROOT/f'assets/lunatic/models/item/lampad_star_{color}.json';p.write_text(json.dumps({'parent':'lunatic:item/lampad_star_voxel','textures':{'star':f'lunatic:item/lampad_star_{color}'}},indent=2)+'\n')
 print('Generated 32-grid voxel star:',len(occupied),'voxels;',len(elements),'surface elements')
if __name__=='__main__':generate()
