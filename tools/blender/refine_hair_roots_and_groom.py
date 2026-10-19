"""Continuous scalp-fitting root bed plus varied physical strands over the cards."""
import bpy,math,random,json
from pathlib import Path
from mathutils import Vector
from mathutils.bvhtree import BVHTree
ROOT=Path('D:/雅思');s=bpy.data.scenes['IELTS_Speaking_Form'];bpy.context.window.scene=s;s.frame_set(1);rig=bpy.data.objects['Examiner_Rig'];rnd=random.Random(881916)
hair=bpy.data.objects['Examiner_Hair'];body=bpy.data.objects['Examiner_Body'];center=Vector((0,-.025,1.49))
for name in ['Examiner_HairRootCap','Examiner_HairStrands']:
    old=bpy.data.objects.get(name)
    if old:bpy.data.objects.remove(old,do_unlink=True)
def bind(o):
    g=o.vertex_groups.new(name='head');g.add(list(range(len(o.data.vertices))),1,'REPLACE');m=o.modifiers.new('Head deformation','ARMATURE');m.object=rig;o.parent=rig
states=[(m,m.show_viewport) for m in body.modifiers if m.type in {'ARMATURE','MASK'}]
for m,_ in states:m.show_viewport=False
bpy.context.view_layer.update();ev=body.evaluated_get(bpy.context.evaluated_depsgraph_get());me=ev.to_mesh();tree=BVHTree.FromPolygons([v.co.copy() for v in me.vertices],[list(p.vertices) for p in me.polygons]);ev.to_mesh_clear()
for m,v in states:m.show_viewport=v
def scalp(p):
    d=(p-center).normalized();hit,n,i,dist=tree.ray_cast(center,d,.3)
    if hit is not None and (p-center).length<(hit-center).length+.0018:p=hit+d*.0018
    return p
verts=[tuple(scalp(Vector((0,-.025,1.618))))];faces=[];cols=128;rows=32
for row in range(1,rows+1):
    for col in range(cols):
        phi=col/cols*2*math.pi;front=max(0,-math.sin(phi));theta=(.95-.35*front**4)*(row/rows)
        p=center+Vector((.086*math.sin(theta)*math.cos(phi),.105*math.sin(theta)*math.sin(phi),.127*math.cos(theta)));verts.append(tuple(scalp(p)))
for col in range(cols):faces.append((0,1+col,1+(col+1)%cols))
for row in range(rows-1):
    for col in range(cols):
        a=1+row*cols+col;b=1+row*cols+(col+1)%cols;faces.append((a,b,b+cols,a+cols))
mesh=bpy.data.meshes.new('IELTS_ContinuousHairRootBed');mesh.from_pydata(verts,[],faces);mesh.update();cap=bpy.data.objects.new('Examiner_HairRootCap',mesh);s.collection.objects.link(cap);bind(cap)
mat=bpy.data.materials.new('IELTS_DarkerBlondeRoots');mat.use_nodes=True;p=next(n for n in mat.node_tree.nodes if n.type=='BSDF_PRINCIPLED');p.inputs['Base Color'].default_value=(.12,.060,.021,1);p.inputs['Roughness'].default_value=.55;mesh.materials.append(mat)
for p in mesh.polygons:p.use_smooth=True
cap['asset_source']='Original scalp-fitting continuous root bed; one welded crown pole'
curve=bpy.data.curves.new('IELTS_VariedPhysicalHair','CURVE');curve.dimensions='3D';curve.resolution_u=1;curve.bevel_depth=.000032;curve.bevel_resolution=0
for i in range(12):
    m=bpy.data.materials.new('IELTS_GroomStrand_'+str(i));m.use_nodes=True;p=next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED');t=i/11
    p.inputs['Base Color'].default_value=(.245+.06*t,.125+.045*t,.044+.025*t,1);p.inputs['Roughness'].default_value=.47
    if 'Anisotropic IOR Level' in p.inputs:p.inputs['Anisotropic IOR Level'].default_value=.75
    curve.materials.append(m)
count=len(hair.data.vertices)//99;total=0
for card in range(count):
    path=[hair.data.vertices[card*99+j*3+1].co.copy() for j in range(33)]
    for strand in range(6):
        sp=curve.splines.new('POLY');sp.points.add(16);sp.material_index=rnd.randrange(12);offset=rnd.uniform(-.0015,.0015);phase=rnd.uniform(0,6.28);lift=rnd.uniform(.0003,.0022);end=rnd.uniform(-.004,.003)
        for index,(pnt,j) in enumerate(zip(sp.points,range(0,33,2))):
            t=j/32;p=path[j];tangent=(path[min(32,j+1)]-path[max(0,j-1)]).normalized();normal=Vector((p.x,(p.y+.025),max(0,p.z-1.50))).normalized();cross=tangent.cross(normal).normalized()
            q=p+cross*(offset+.0004*math.sin(t*11+phase)*math.sin(math.pi*t))+normal*(.0003+lift*math.sin(math.pi*t)**2)+Vector((0,0,end*t*t))
            pnt.co=(*q,1);pnt.radius=max(.06,1-.94*t**4)
        total+=1
# A few restrained flyaways break the perfect sheet silhouette.
for i in range(60):
    card=rnd.randrange(count);path=[hair.data.vertices[card*99+j*3+1].co.copy() for j in range(33)];sp=curve.splines.new('POLY');sp.points.add(12);sp.material_index=rnd.randrange(12)
    for j,pnt in enumerate(sp.points):
        t=j/12;p=path[4+j*2];n=Vector((p.x,p.y+.025,max(.01,p.z-1.50))).normalized();q=p+n*(.0004+.003*math.sin(math.pi*t));pnt.co=(*q,1);pnt.radius=max(.06,1-t*.94)
ob=bpy.data.objects.new('Examiner_HairStrands',curve);s.collection.objects.link(ob);bpy.ops.object.select_all(action='DESELECT');ob.select_set(True);bpy.context.view_layer.objects.active=ob;bpy.ops.object.convert(target='MESH');ob=bpy.context.object;bind(ob)
for polygon in ob.data.polygons:polygon.use_smooth=True
for m in hair.data.materials:
    for n in m.node_tree.nodes:
        if n.type=='BUMP':n.inputs['Strength'].default_value=.06
info={'rootBedVertices':len(mesh.vertices),'rootBedConnectedCrown':True,'physicalStrands':total,'flyaways':60,'groomMeshVertices':len(ob.data.vertices),'hairPhysics':False,'runtimeOptimizationPending':True,'source':'Original procedural root bed and physical strand overlay'}
(ROOT/'assets/characters/examiner/hair-root-groom.json').write_text(json.dumps(info,indent=2),encoding='utf-8');bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'blender/speaking_room.blend'));print('ROOT_GROOM_REFINEMENT',json.dumps(info))
