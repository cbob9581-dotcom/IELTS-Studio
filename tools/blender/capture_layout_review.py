"""Render Function review views from actual layout geometry, then restore the live scene."""
import bpy
import bmesh
import json
import math
from pathlib import Path
from mathutils import Vector

ROOT=Path('D:/雅思')
PACK=ROOT/'rooms/speaking'
s=bpy.context.scene
main=bpy.data.objects['IELTS_ProductionCamera']
original_camera=s.camera
original_engine=s.render.engine
original_res=(s.render.resolution_x,s.render.resolution_y,s.render.resolution_percentage)
hidden={o.name:o.hide_render for o in s.objects}

def render(path):
    s.render.filepath=str(PACK/path)
    bpy.ops.render.render(write_still=True)

def camera(name,loc,target,ortho=None,lens=24):
    d=bpy.data.cameras.new(name);o=bpy.data.objects.new(name,d)
    s.collection.objects.link(o);o.location=loc
    o.rotation_euler=(Vector(target)-o.location).to_track_quat('-Z','Y').to_euler()
    d.lens=lens
    if ortho:
        enum=[e.identifier for e in d.bl_rna.properties['type'].enum_items]
        d.type=next(e for e in enum if e=='ORTHO');d.ortho_scale=ortho
    return o

# Perspective view: camera entirely inside the sealed room.
s.render.resolution_x=1280;s.render.resolution_y=800;s.render.resolution_percentage=100
s.camera=bpy.data.objects['IELTS_OverviewCamera']
render('renders/layout-overview.png')

# Section copies are derived from evaluated meshes, never generic object boxes.
section=bpy.data.collections.new('IELTS_PLAN_SECTION_TEMP')
s.collection.children.link(section)
deps=bpy.context.evaluated_depsgraph_get()
for src in list(s.objects):
    if src.type!='MESH' or src.hide_render or 'Ceiling' in src.name: continue
    mesh=bpy.data.meshes.new_from_object(src.evaluated_get(deps))
    bm=bmesh.new();bm.from_mesh(mesh);bm.transform(src.matrix_world)
    if bm.verts:
        result=bmesh.ops.bisect_plane(bm,geom=list(bm.verts)+list(bm.edges)+list(bm.faces),dist=.00001,plane_co=(0,0,1.2),plane_no=(0,0,1),clear_outer=True,clear_inner=False)
        boundary=[e for e in bm.edges if e.is_boundary]
        if boundary:
            try:bmesh.ops.holes_fill(bm,edges=boundary,sides=0)
            except RuntimeError:pass
    bm.to_mesh(mesh);bm.free()
    ob=bpy.data.objects.new('PLAN_'+src.name,mesh);section.objects.link(ob)
for o in list(s.objects):
    if o.type=='MESH' and not o.name.startswith('PLAN_'):o.hide_render=True
try:s.render.engine='BLENDER_WORKBENCH'
except TypeError as ex:raise RuntimeError('Workbench unavailable: '+str(ex))
sh=s.display.shading
sh.light=next(x.identifier for x in sh.bl_rna.properties['light'].enum_items if x.identifier=='FLAT')
sh.color_type=next(x.identifier for x in sh.bl_rna.properties['color_type'].enum_items if x.identifier=='SINGLE')
sh.single_color=(.85,.85,.85)
sh.background_type=next(x.identifier for x in sh.bl_rna.properties['background_type'].enum_items if x.identifier=='WORLD')
s.world.color=(1,1,1)
sh.show_shadows=False;sh.show_cavity=True;sh.show_object_outline=True;sh.object_outline_color=(0,0,0)
s.render.resolution_x=1000;s.render.resolution_y=1000
top=camera('IELTS_PlanCamera',(.4,0,8),(.4,0,0),6.5)
s.camera=top;render('plans/floor-plan-mesh-render.png')

for o in section.objects:o.hide_render=True
for o in s.objects:
    if o.type=='MESH' and any(x in o.name for x in ['Ceiling','Wall_','Window_','Door_','Corridor_']):o.hide_render=False
top.location=(.4,0,-3);top.rotation_euler=(Vector((.4,0,2.8))-top.location).to_track_quat('-Z','Y').to_euler()
render('plans/ceiling-plan-mesh-render.png')

for o in s.objects:
    if o.type=='MESH':o.hide_render=True
for wall,label,loc in [('West','west',(-6,0,1.4)),('East','east',(6,0,1.4))]:
    for o in s.objects:
        if o.type=='MESH':
            o.hide_render=not (o.name=='IELTS_Wall_'+wall or (wall=='West' and 'Window_' in o.name) or (wall=='East' and 'Door_' in o.name))
    top.location=loc;top.rotation_euler=(Vector((0,0,1.4))-top.location).to_track_quat('-Z','Y').to_euler()
    top.data.ortho_scale=5.7;s.render.resolution_x=1200;s.render.resolution_y=700
    render('plans/'+label+'-elevation-mesh-render.png')

# Isolated procedural furniture references for asset review.
for prefix,name in [('IELTS_Desk_','desk'),('IELTS_ExaminerChair_','chair')]:
    for o in s.objects:
        if o.type=='MESH':o.hide_render=not o.name.startswith(prefix)
    center=Vector((0,0,.38)) if name=='desk' else Vector((0,.95,.48))
    top.location=center+Vector((2,-2,1.4));top.rotation_euler=(center-top.location).to_track_quat('-Z','Y').to_euler()
    top.data.ortho_scale=2.0 if name=='desk' else 1.35
    s.render.resolution_x=900;s.render.resolution_y=900
    render('images/'+name+'-reference.png')

for ob in list(section.objects):
    mesh=ob.data;bpy.data.objects.remove(ob,do_unlink=True);bpy.data.meshes.remove(mesh)
bpy.data.collections.remove(section)
for o in s.objects:
    if o.name in hidden:o.hide_render=hidden[o.name]
s.render.engine=original_engine;s.camera=main
s.render.resolution_x,s.render.resolution_y,s.render.resolution_percentage=original_res
s.render.filepath=str(PACK/'renders/layout-production.png')
layout=json.loads((PACK/'room-layout.json').read_text(encoding='utf-8'))
layout['productionCamera']['target']=[0,.95,1.08]
layout['geometryObjects']=[dict(id=o.name,location=list(o.location),dimensions=list(o.dimensions),rotationRadians=list(o.rotation_euler),status=o.get('asset_status','layout_geometry')) for o in s.objects if o.type=='MESH']
(PACK/'room-layout.json').write_text(json.dumps(layout,ensure_ascii=False,indent=2),encoding='utf-8')
bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'blender/speaking_layout.blend'))
print('REVIEW_CAPTURE_COMPLETE',len(layout['geometryObjects']))
