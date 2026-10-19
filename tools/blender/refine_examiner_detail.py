"""Add restrained facial shading and actual hair fiber geometry to the examiner."""
import bpy,math,random,json
from pathlib import Path
from mathutils import Vector
from mathutils.bvhtree import BVHTree
ROOT=Path('D:/雅思');s=bpy.data.scenes['IELTS_Speaking_Form'];bpy.context.window.scene=s;s.frame_set(1)
body=bpy.data.objects['Examiner_Body'];rig=bpy.data.objects['Examiner_Rig']
def shader(obj):
    m=obj.active_material
    return m,next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED')

m,p=shader(body);nodes=m.node_tree.nodes;links=m.node_tree.links
base=next(no for no in nodes if no.type=='TEX_IMAGE' and no.image and 'diffuse' in no.image.name).outputs['Color']
coord=nodes.new('ShaderNodeTexCoord')
noise=nodes.new('ShaderNodeTexNoise');noise.inputs['Scale'].default_value=1600;noise.inputs['Detail'].default_value=2.5
links.new(coord.outputs['Object'],noise.inputs['Vector'])
bump=nodes.new('ShaderNodeBump');bump.inputs['Strength'].default_value=.15;bump.inputs['Distance'].default_value=.00009
links.new(noise.outputs['Fac'],bump.inputs['Height']);links.new(bump.outputs['Normal'],p.inputs['Normal'])
ramp=nodes.new('ShaderNodeValToRGB');ramp.color_ramp.elements[0].color=(.40,.40,.40,1);ramp.color_ramp.elements[1].color=(.57,.57,.57,1)
links.new(noise.outputs['Fac'],ramp.inputs[0]);links.new(ramp.outputs[0],p.inputs['Roughness'])
p.inputs['Subsurface Weight'].default_value=.055
if 'Subsurface Scale' in p.inputs:p.inputs['Subsurface Scale'].default_value=.003

# Lip tint follows existing anatomical vertex groups through all expressions.
name='IELTS_FacialTone'
attr=body.data.color_attributes.get(name) or body.data.color_attributes.new(name=name,type='FLOAT_COLOR',domain='POINT')
lip=body.vertex_groups.get('lips')
for i,v in enumerate(body.data.vertices):
    w=sum(g.weight for g in v.groups if lip and g.group==lip.index)
    attr.data[i].color=(.46,.16,.135,min(.24,w*.24))
vc=nodes.new('ShaderNodeVertexColor');vc.layer_name=name
mix=nodes.new('ShaderNodeMixRGB');links.new(vc.outputs['Alpha'],mix.inputs[0]);links.new(base,mix.inputs[1]);links.new(vc.outputs['Color'],mix.inputs[2]);links.new(mix.outputs[0],p.inputs['Base Color'])

m,p=shader(bpy.data.objects['Examiner_Eyes']);p.inputs['Roughness'].default_value=.12
eye_path=ROOT/'.tools/mpfb-user/data/eyes/materials/bluegreen_eye.png'
for tex in m.node_tree.nodes:
    if tex.type=='TEX_IMAGE' and tex.image and '_eye' in tex.image.name:
        tex.image=bpy.data.images.load(str(eye_path),check_existing=True)
if 'Coat Weight' in p.inputs:p.inputs['Coat Weight'].default_value=.45;p.inputs['Coat Roughness'].default_value=.035
p.inputs['IOR'].default_value=1.38
eye_source=next(n for n in m.node_tree.nodes if n.type=='TEX_IMAGE' and n.image and '_eye' in n.image.name)
tone=m.node_tree.nodes.new('ShaderNodeHueSaturation');tone.inputs['Saturation'].default_value=.35;tone.inputs['Value'].default_value=.9
m.node_tree.links.new(eye_source.outputs['Color'],tone.inputs['Color']);m.node_tree.links.new(tone.outputs['Color'],p.inputs['Base Color'])
m,p=shader(bpy.data.objects['Examiner_Eyebrows'])
source=next(no for no in m.node_tree.nodes if no.type=='TEX_IMAGE' and no.image).outputs['Color']
mix=m.node_tree.nodes.new('ShaderNodeMixRGB');mix.blend_type='MULTIPLY';mix.inputs[0].default_value=1;mix.inputs[2].default_value=(.65,.46,.29,1)
m.node_tree.links.new(source,mix.inputs[1]);m.node_tree.links.new(mix.outputs[0],p.inputs['Base Color'])

hair=bpy.data.objects['Examiner_Hair'];m,p=shader(hair)
normal_path=ROOT/'.tools/mpfb-user/data/hair/elvs_short_daisy_hair/daisyhair1_normals.png'
if normal_path.exists():
    tex=m.node_tree.nodes.new('ShaderNodeTexImage');tex.image=bpy.data.images.load(str(normal_path),check_existing=True);tex.image.colorspace_settings.name='Non-Color'
    normal=m.node_tree.nodes.new('ShaderNodeNormalMap');normal.inputs['Strength'].default_value=.45
    normal.inputs['Strength'].default_value=.18
    m.node_tree.links.new(tex.outputs['Color'],normal.inputs['Color']);m.node_tree.links.new(normal.outputs[0],p.inputs['Normal'])

# Project fibers onto the real evaluated hair cap, with armature deformation disabled.
states=[(mod,mod.show_viewport) for mod in hair.modifiers if mod.type=='ARMATURE']
for mod,_ in states:mod.show_viewport=False
bpy.context.view_layer.update();dep=bpy.context.evaluated_depsgraph_get();ev=hair.evaluated_get(dep);mesh=ev.to_mesh()
verts=[v.co.copy() for v in mesh.vertices];faces=[list(poly.vertices) for poly in mesh.polygons]
bvh=BVHTree.FromPolygons(verts,faces)
ev.to_mesh_clear()
for mod,enabled in states:mod.show_viewport=enabled
bpy.context.view_layer.update()
old=bpy.data.objects.get('Examiner_HairFibers')
if old:bpy.data.objects.remove(old,do_unlink=True)
curve=bpy.data.curves.new('Examiner_HairFiberGeometry','CURVE');curve.dimensions='3D';curve.resolution_u=2;curve.bevel_depth=.000085;curve.bevel_resolution=0
rand=random.Random(8319)
for i in range(8):
    mat=bpy.data.materials.new('IELTS_HairFiber_'+str(i));mat.use_nodes=True
    pp=next(n for n in mat.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
    t=i/7;pp.inputs['Base Color'].default_value=(.15+t*.34,.075+t*.25,.025+t*.12,1);pp.inputs['Roughness'].default_value=.34
    curve.materials.append(mat)
center=Vector((0,-.032,rig.data.bones['eye.L'].head_local.z+.035))
count=0
for i in range(700):
    phi=rand.uniform(-math.pi,math.pi);start=rand.uniform(.06,.28);end=rand.uniform(1.35,2.15)
    points=[]
    for j in range(14):
        t=j/13;theta=start+(end-start)*t;az=phi+.30*t
        direction=Vector((math.sin(theta)*math.cos(az),math.sin(theta)*math.sin(az),math.cos(theta)))
        hit,normal,idx,distance=bvh.ray_cast(center+direction*.32,-direction,.50)
        if hit is not None and normal.dot(direction)>.05:
            points.append(hit+normal*(.00025+rand.random()*.00025))
    if len(points)<7:continue
    sp=curve.splines.new('NURBS');sp.points.add(len(points)-1);sp.order_u=3;sp.use_endpoint_u=True;sp.material_index=rand.randrange(8)
    for j,(pnt,co) in enumerate(zip(sp.points,points)):
        pnt.co=(*co,1);pnt.radius=max(.10,1-(j/max(1,len(points)-1))*.84)
    count+=1
obj=bpy.data.objects.new('Examiner_HairFibers',curve);s.collection.objects.link(obj)
bpy.ops.object.select_all(action='DESELECT');obj.select_set(True);bpy.context.view_layer.objects.active=obj
bpy.ops.object.convert(target='MESH');obj=bpy.context.object
group=obj.vertex_groups.new(name='head');group.add(list(range(len(obj.data.vertices))),1,'REPLACE')
arm=obj.modifiers.new('Head bone deformation','ARMATURE');arm.object=rig;obj.parent=rig
obj['asset_source']='Project-authored fibers projected on modified Elvaerwyn Short Daisy Hair (CC-BY)';obj['fiber_count']=count
for poly in obj.data.polygons:poly.use_smooth=True

# A closer soft key and lower ambient level reveal facial form and eye highlights.
bg=next(n for n in s.world.node_tree.nodes if n.type=='BACKGROUND');bg.inputs['Strength'].default_value=.12
for o in s.objects:
    if o.type=='LIGHT':
        logical=o.get('logical_id','')
        if logical=='IELTS_CeilingLight':o.data.energy=70
        if logical=='IELTS_WindowLight':o.data.energy=155
        if logical=='IELTS_FaceFill':o.data.energy=10
light=bpy.data.objects.get('IELTS_PortraitKey')
if not light:
    d=bpy.data.lights.new('IELTS_PortraitKey','AREA');light=bpy.data.objects.new('IELTS_PortraitKey',d);s.collection.objects.link(light)
head_world=rig.matrix_world@rig.pose.bones['head'].head
light.location=(-.85,-.15,1.8);light.rotation_euler=(head_world-light.location).to_track_quat('-Z','Y').to_euler();light.data.energy=38;light.data.size=.75;light.data.color=(1,.91,.82)
info={'fiberSplines':count,'fiberMeshVertices':len(obj.data.vertices),'skinPoreReliefMeters':.00009,'skinSubsurfaceScaleMeters':.003,'lipTintMaxWeight':.24,'appearanceStage':'review refinement v2'}
(ROOT/'assets/characters/examiner/detail-settings.json').write_text(json.dumps(info,indent=2),encoding='utf-8')
for image in bpy.data.images:
    if image.source=='FILE' and image.users>0:
        try:image.pack()
        except RuntimeError:pass
bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'blender/speaking_room.blend'))
print('DETAIL_REFINEMENT',json.dumps(info))
