"""Create a separate detailed environment from the approved IELTS layout."""
import bpy
import json
from pathlib import Path
from datetime import datetime,timezone
from mathutils import Vector
ROOT=Path('D:/雅思');PACK=ROOT/'rooms/speaking'
if bpy.data.scenes.get('IELTS_Speaking_Form'):
    raise RuntimeError('Form scene already exists; inspect before rebuilding.')
source=bpy.data.scenes['IELTS_Speaking_Layout']
scene=bpy.data.scenes.new('IELTS_Speaking_Form')
scene.world=source.world.copy()
scene.unit_settings.system=source.unit_settings.system
scene.unit_settings.scale_length=1
objects={};materials={}
for old in source.objects:
    if old.name.startswith('IELTS_PlanCamera'):continue
    obj=old.copy()
    if old.data:obj.data=old.data.copy()
    scene.collection.objects.link(obj);objects[old.name]=obj;obj['logical_id']=old.name
    if obj.type=='MESH':
        for slot in obj.material_slots:
            if slot.material:
                key=slot.material.name
                if key not in materials:materials[key]=slot.material.copy()
                slot.material=materials[key]
scene.camera=objects['IELTS_ProductionCamera']
bpy.context.window.scene=scene
scene.render.resolution_x=1600;scene.render.resolution_y=900
scene.render.resolution_percentage=100
scene.render.use_compositing=False
scene.render.engine=source.render.engine
scene.view_settings.view_transform=source.view_settings.view_transform
scene['stage']='Form environment in progress; character not yet integrated'

def principled(mat):return next(n for n in mat.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
def add_noise(mat,scale):
    n=mat.node_tree.nodes.new('ShaderNodeTexNoise');n.inputs['Scale'].default_value=scale
    n.inputs['Detail'].default_value=2
    coord=mat.node_tree.nodes.new('ShaderNodeTexCoord')
    mat.node_tree.links.new(coord.outputs['Generated'],n.inputs['Vector'])
    return n

oak=materials['IELTS_LightOak'];p=principled(oak)
nodes=oak.node_tree.nodes;links=oak.node_tree.links
noise=nodes.new('ShaderNodeTexNoise');noise.inputs['Scale'].default_value=4.5;noise.inputs['Detail'].default_value=3
coord=nodes.new('ShaderNodeTexCoord');mapping=nodes.new('ShaderNodeVectorMath');mapping.operation='MULTIPLY';mapping.inputs[1].default_value=(1,45,3)
links.new(coord.outputs['Generated'],mapping.inputs[0]);links.new(mapping.outputs[0],noise.inputs['Vector'])
ramp=nodes.new('ShaderNodeValToRGB')
ramp.color_ramp.elements[0].position=.18;ramp.color_ramp.elements[0].color=(.26,.155,.075,1)
ramp.color_ramp.elements[1].position=.83;ramp.color_ramp.elements[1].color=(.58,.40,.215,1)
links.new(noise.outputs['Fac'],ramp.inputs[0]);links.new(ramp.outputs['Color'],p.inputs['Base Color'])
p.inputs['Roughness'].default_value=.42
bump=nodes.new('ShaderNodeBump');bump.inputs['Strength'].default_value=.10;bump.inputs['Distance'].default_value=.001
links.new(noise.outputs['Fac'],bump.inputs['Height']);links.new(bump.outputs['Normal'],p.inputs['Normal'])

for name,color,scale,strength,distance in [
    ('IELTS_ChairFabric',(.055,.075,.10,1),220,.16,.0006),
    ('IELTS_Carpet',(.115,.125,.135,1),350,.25,.0007),
    ('IELTS_WarmPlaster',(.57,.55,.50,1),150,.10,.0004)]:
    m=materials[name];pp=principled(m);pp.inputs['Base Color'].default_value=color;pp.inputs['Roughness'].default_value=.82
    n=add_noise(m,scale);b=m.node_tree.nodes.new('ShaderNodeBump');b.inputs['Strength'].default_value=strength;b.inputs['Distance'].default_value=distance
    m.node_tree.links.new(n.outputs['Fac'],b.inputs['Height']);m.node_tree.links.new(b.outputs['Normal'],pp.inputs['Normal'])
principled(materials['IELTS_DarkMetal']).inputs['Metallic'].default_value=.75
principled(materials['IELTS_DarkMetal']).inputs['Roughness'].default_value=.32

# The visible luminaire and the light object share a stable fixture identifier.
panel=materials['IELTS_Paper'].copy();panel.name='IELTS_Form_Luminaire'
pp=principled(panel);pp.inputs['Base Color'].default_value=(.86,.84,.74,1)
pp.inputs['Emission Color'].default_value=(1,.92,.78,1);pp.inputs['Emission Strength'].default_value=1.5
objects['IELTS_Ceiling_Light_Panel'].data.materials.clear();objects['IELTS_Ceiling_Light_Panel'].data.materials.append(panel)
objects['IELTS_Ceiling_Light_Panel']['fixture_id']='ceiling-main'

# Glass is a thin transparent surface; wall aperture and frame remain geometric.
glass=bpy.data.materials.new('IELTS_Form_WindowGlass');glass.use_nodes=True
gn=glass.node_tree.nodes;gl=glass.node_tree.links;gn.clear()
out=gn.new('ShaderNodeOutputMaterial');transparent=gn.new('ShaderNodeBsdfTransparent');transparent.inputs[0].default_value=(.91,.96,1,1)
gl.new(transparent.outputs[0],out.inputs['Surface'])
bpy.ops.mesh.primitive_cube_add(size=1,location=(-2.045,.35,1.65))
pane=bpy.context.object;pane.name='IELTS_Form_WindowGlass';pane.dimensions=(.006,1.78,1.28)
bpy.ops.object.transform_apply(location=False,rotation=False,scale=True);pane.data.materials.append(glass)
pane['opening_id']='W01'

# Soft outside backing has a gradient but is kept physically beyond the aperture.
outside=objects['IELTS_Exterior_Lightwell'];m=bpy.data.materials.new('IELTS_Form_ExteriorBacking');m.use_nodes=True
n=m.node_tree.nodes;l=m.node_tree.links;pp=principled(m)
tex=n.new('ShaderNodeTexCoord');sep=n.new('ShaderNodeSeparateXYZ');ramp=n.new('ShaderNodeValToRGB')
l.new(tex.outputs['Generated'],sep.inputs[0]);l.new(sep.outputs['Z'],ramp.inputs[0])
ramp.color_ramp.elements[0].color=(.26,.32,.27,1);ramp.color_ramp.elements[1].color=(.56,.68,.78,1)
l.new(ramp.outputs[0],pp.inputs['Base Color']);l.new(ramp.outputs[0],pp.inputs['Emission Color']);pp.inputs['Emission Strength'].default_value=.35
outside.data.materials.clear();outside.data.materials.append(m)

fixtures=[]
for key,power in [('IELTS_WindowLight',260),('IELTS_CeilingLight',110),('IELTS_FaceFill',28)]:
    o=objects[key];o.data.energy=power
    fixtures.append({'id':key,'location':list(o.location),'powerWatts':power,'color':list(o.data.color),'sizeMeters':o.data.size})
for screen in bpy.data.screens:
    for area in screen.areas:
        if area.type=='VIEW_3D':area.spaces.active.region_3d.view_perspective='CAMERA'
scene.render.filepath=str(PACK/'renders/form-environment-production.png')
bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'blender/speaking_room.blend'))
(PACK/'fixture-manifest.json').write_text(json.dumps({'roomId':'speaking','fixtures':fixtures},ensure_ascii=False,indent=2),encoding='utf-8')

reviews=json.loads((PACK/'milestone-reviews.json').read_text(encoding='utf-8'))
reviews['function'].update(status='approved',approvedBy='user in chat',approvedAt=datetime.now(timezone.utc).isoformat(),notes='User replied ok after reviewing layout preview and examiner reference. No paid asset generation approved or submitted.')
(PACK/'milestone-reviews.json').write_text(json.dumps(reviews,ensure_ascii=False,indent=2),encoding='utf-8')
props=json.loads((PACK/'props.json').read_text(encoding='utf-8'))
for a in props['assets']:
    if a['id'] in ['desk','chair','stationery','examiner-envelope']:
        a['image'].update(approved=True,approvedBy='user in chat: layout and examiner appearance')
(PACK/'props.json').write_text(json.dumps(props,ensure_ascii=False,indent=2),encoding='utf-8')
print('FORM_ENVIRONMENT_SAVED',bpy.data.filepath)
