"""Project-authored narrow hair cards: side part, chin bob, connected short nape."""
import bpy,math,random,json
from pathlib import Path
from mathutils import Vector
from mathutils.bvhtree import BVHTree
ROOT=Path('D:/雅思');s=bpy.data.scenes['IELTS_Speaking_Form'];bpy.context.window.scene=s;s.frame_set(1)
rig=bpy.data.objects['Examiner_Rig'];body=bpy.data.objects['Examiner_Body'];rnd=random.Random(261015)
for name in ['Examiner_Hair','Examiner_HairFibers','Examiner_HairCards','Examiner_HairStrands']:
    ob=bpy.data.objects.get(name)
    if ob:bpy.data.objects.remove(ob,do_unlink=True)
states=[(m,m.show_viewport) for m in body.modifiers if m.type in {'ARMATURE','MASK'}]
for mod,_ in states:mod.show_viewport=False
bpy.context.view_layer.update();ev=body.evaluated_get(bpy.context.evaluated_depsgraph_get());me=ev.to_mesh()
tree=BVHTree.FromPolygons([v.co.copy() for v in me.vertices],[list(p.vertices) for p in me.polygons]);ev.to_mesh_clear()
for mod,enabled in states:mod.show_viewport=enabled
center=Vector((0,-.025,1.49))
def safe(p):
    if p.z>1.465:
        d=(p-center).normalized();hit,normal,idx,dist=tree.ray_cast(center,d,.30)
        if hit is not None and (p-center).length<(hit-center).length+.004:p=hit+d*.004
    return p
def side_path(sign,y,layer,phase):
    points=[];q=(y+.025)/.112;rise=math.sqrt(max(.04,1-q*q));end=1.382+.012*max(0,q)+.005*math.sin(phase)+layer*.012
    for j in range(33):
        t=j/32;u=min(1,t/.68);a=u*math.pi/2
        x=.014*math.cos(a)+sign*(.097+layer*.0015)*rise*math.sin(a)
        yy=y+.009*math.sin(a)
        z=1.50+.127*rise*math.cos(a)
        if t>.68:
            v=(t-.68)/.32;z=1.50+(end-1.50)*v;x-=sign*(.009+layer*.006)*rise*v*v;yy-=.004*v*v
        # Every decorative offset has zero position AND zero derivative at root.
        root_falloff=math.sin(math.pi*t)**2
        x+=.0008*math.sin(t*8+phase)*root_falloff
        lift=layer*.004*root_falloff
        if layer:x+=sign*.002*math.sin(t*8+phase)*root_falloff
        points.append(safe(Vector((x+sign*lift,yy,z)))+Vector((0,0,lift)))
    return points
def back_path(x,layer,phase):
    pts=[];rise=math.sqrt(max(.05,1-(x/.10)**2))
    for j in range(33):
        t=j/32;u=min(1,t/.68);a=u*math.pi/2
        p=Vector((x,.003+.090*rise*math.sin(a),1.50+.126*rise*math.cos(a)))
        if t>.68:
            v=(t-.68)/.32;p.z=1.50+(1.395+.003*math.sin(phase)-1.50)*v;p.y-=.012*v*v
        pts.append(safe(p)+Vector((0,layer*.001,0)))
    return pts
def crown_back_path(phi,layer,phase):
    pts=[];rise=math.sqrt(1-((.010+.025)/.112)**2)
    for j in range(33):
        t=j/32;u=min(1,t/.68);a=u*math.pi/2
        p=Vector((.014*math.cos(a)+(.097+layer*.0015)*rise*math.cos(phi)*math.sin(a),.010+(.009+.087*math.sin(phi))*math.sin(a),1.50+.127*rise*math.cos(a)))
        if t>.68:
            v=(t-.68)/.32;p.z=1.50+(1.38575+.00825*math.sin(phi)+.004*math.sin(phase)+layer*.012-1.50)*v
            p.x-=(.009+layer*.006)*rise*math.cos(phi)*v*v;p.y-=(.004+.006*math.sin(phi))*v*v
        root_falloff=math.sin(math.pi*t)**2;lift=layer*.004*root_falloff
        p.x+=.0005*math.sin(t*7+phase)*root_falloff+lift*math.cos(phi);p.y+=lift*math.sin(phi);p.z+=lift
        pts.append(safe(p))
    return pts
materials=[]
for i in range(9):
    m=bpy.data.materials.new('IELTS_RebuiltHair_'+str(i));m.use_nodes=True;n=m.node_tree.nodes;l=m.node_tree.links;p=next(x for x in n if x.type=='BSDF_PRINCIPLED')
    t=i/8;p.inputs['Base Color'].default_value=(.24+t*.08,.125+t*.055,.045+t*.028,1);p.inputs['Roughness'].default_value=.46
    if 'Anisotropic IOR Level' in p.inputs:p.inputs['Anisotropic IOR Level'].default_value=.65
    tex=n.new('ShaderNodeTexCoord');mapping=n.new('ShaderNodeVectorMath');mapping.operation='MULTIPLY';mapping.inputs[1].default_value=(95,1.5,1);l.new(tex.outputs['UV'],mapping.inputs[0])
    noise=n.new('ShaderNodeTexNoise');noise.inputs['Scale'].default_value=6;noise.inputs['Detail'].default_value=2;l.new(mapping.outputs[0],noise.inputs['Vector'])
    bump=n.new('ShaderNodeBump');bump.inputs['Strength'].default_value=.18;bump.inputs['Distance'].default_value=.000035;l.new(noise.outputs['Fac'],bump.inputs['Height']);l.new(bump.outputs[0],p.inputs['Normal'])
    separate=n.new('ShaderNodeSeparateXYZ');l.new(tex.outputs['UV'],separate.inputs[0])
    remain=n.new('ShaderNodeMath');remain.operation='SUBTRACT';remain.inputs[0].default_value=1;l.new(separate.outputs['Y'],remain.inputs[1])
    fade=n.new('ShaderNodeMath');fade.operation='MULTIPLY';fade.inputs[1].default_value=5;fade.use_clamp=True;l.new(remain.outputs[0],fade.inputs[0]);l.new(fade.outputs[0],p.inputs['Alpha'])
    materials.append(m)
verts=[];faces=[];uvs=[];mids=[];paths=[]
def card(path,width,mat):
    start=len(verts);previous=None
    for j,p in enumerate(path):
        t=j/(len(path)-1);tangent=(path[min(j+1,len(path)-1)]-path[max(j-1,0)]).normalized()
        normal=Vector((p.x/.01,(p.y+.025)/.012,max(.0,p.z-1.50)/.016)).normalized();across=tangent.cross(normal).normalized()
        if previous is not None and across.dot(previous)<0:across=-across
        previous=across
        taper=min(1,.18+t*7)*max(.015,min(1,(1-t)*8))
        for k in range(3):
            u=k/2;co=p+across*((u-.5)*width*taper)+normal*(.00025*(1-(u*2-1)**2));verts.append(tuple(co));uvs.append((u,t))
    for j in range(len(path)-1):
        for k in range(2):
            a=start+j*3+k;faces.append((a,a+1,a+4,a+3));mids.append(mat)
    paths.append(path)
for layer,count in [(0,150),(1,190)]:
    for sign in [-1,1]:
        for i in range(count):
            y=-.113+(i+.5)/count*.123+rnd.uniform(-.0001,.0001)
            phase=(i//9)*.6+sign*.2+rnd.uniform(-.05,.05)
            card(side_path(sign,y,layer,phase),.0044 if layer==0 else .0028,rnd.randrange(9))
    for i in range(170):
        phi=(i+.5)/170*math.pi
        card(crown_back_path(phi,layer,(i//8)*.6+rnd.uniform(-.05,.05)),.0038 if layer==0 else .0026,rnd.randrange(9))
    # Recessed nape underlayer closes the rear center without crossing the crown.
    for i in range(60):
        x=-.059+(i+.5)/60*.118;phase=rnd.uniform(0,6.28);path=[]
        for j in range(33):
            t=j/32;y=(.058+.025*math.sin(math.pi*t/2)-.008*t*t)*math.sqrt(1-(x/.11)**2)
            path.append(Vector((x,y+layer*.0006,1.55-(.155+.002*math.sin(phase))*t)))
        card(path,.0038 if layer==0 else .0027,rnd.randrange(9))
mesh=bpy.data.meshes.new('IELTS_NarrowCardBob');mesh.from_pydata(verts,[],faces);mesh.update()
o=bpy.data.objects.new('Examiner_Hair',mesh);s.collection.objects.link(o)
for m in materials:mesh.materials.append(m)
uv=mesh.uv_layers.new(name='HairFlow')
for poly,mat in zip(mesh.polygons,mids):
    poly.use_smooth=True;poly.material_index=mat
    for loop in poly.loop_indices:uv.data[loop].uv=uvs[mesh.loops[loop].vertex_index]
def bind(obj):
    vg=obj.vertex_groups.new(name='head');vg.add(list(range(len(obj.data.vertices))),1,'REPLACE');arm=obj.modifiers.new('Head bone deformation','ARMATURE');arm.object=rig;obj.parent=rig
bind(o);o['asset_source']='Project-authored procedural hair cards, original geometry';o['hair_card_count']=len(paths)
# Fine physical fibers follow the same flow; no wide painted ribbons.
curve=bpy.data.curves.new('IELTS_FineHairStrands','CURVE');curve.dimensions='3D';curve.resolution_u=1;curve.bevel_depth=.000035;curve.bevel_resolution=0
for m in materials:curve.materials.append(m)
for path in paths:
    for repeat in range(2):
        sp=curve.splines.new('POLY');sp.points.add(len(path)-1);sp.material_index=rnd.randrange(9);offset=rnd.uniform(-.0008,.0008)
        for j,(pnt,p) in enumerate(zip(sp.points,path)):
            d=Vector((p.x,p.y+.025,max(0,p.z-1.5))).normalized();q=p+d*.0004+Vector((offset,0,0));pnt.co=(*q,1);pnt.radius=max(.08,1-.92*(j/(len(path)-1))**3)
ob=bpy.data.objects.new('Examiner_HairStrands',curve);s.collection.objects.link(ob);bpy.ops.object.select_all(action='DESELECT');ob.select_set(True);bpy.context.view_layer.objects.active=ob;bpy.ops.object.convert(target='MESH');ob=bpy.context.object;bind(ob)
info={'version':'hair-card-rebuild-v4','cards':len(paths),'cardVertices':len(verts),'cardTriangles':len(faces)*2,'fineStrands':len(paths)*2,'fiberVertices':len(ob.data.vertices),'part':'soft side part','frontEndRestMeters':1.382,'backEndRestMeters':1.395,'provenance':'Original project procedural geometry; previous CC-BY hair retained only in backup'}
(ROOT/'assets/characters/examiner/hair-rebuild.json').write_text(json.dumps(info,indent=2),encoding='utf-8')
bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'blender/speaking_room.blend'));print('REBUILT_HAIR',json.dumps(info))
