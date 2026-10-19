"""Function-stage layout only. Run in Blender; preserves existing scenes."""
import bpy
import json
import math
from pathlib import Path
from mathutils import Vector

ROOT = Path('D:/雅思')
PACK = ROOT / 'rooms/speaking'
NAME = 'IELTS_Speaking_Layout'
if bpy.data.scenes.get(NAME):
    raise RuntimeError('Layout already exists: inspect it before rebuilding.')
for folder in ['backups', 'blender', 'app/modules/speaking', 'app/modules/listening',
               'app/modules/reading', 'app/modules/writing', 'assets/characters/examiner',
               'services/asr', 'services/dialogue', 'services/tts']:
    (ROOT / folder).mkdir(parents=True, exist_ok=True)
backup = ROOT / 'backups/pre-ielts.blend'
if not backup.exists():
    bpy.ops.wm.save_as_mainfile(filepath=str(backup), copy=True)
original_scene = bpy.context.scene.name
scene = bpy.data.scenes.new(NAME)
bpy.context.window.scene = scene
scene.unit_settings.system = 'METRIC'
scene.unit_settings.scale_length = 1
scene.render.resolution_x = 1440
scene.render.resolution_y = 810
scene.render.resolution_percentage = 100
scene.world = bpy.data.worlds.new('IELTS_Soft_World')
scene.world.use_nodes = True
bg = next(n for n in scene.world.node_tree.nodes if n.type == 'BACKGROUND')
bg.inputs['Color'].default_value = (0.78, 0.82, 0.9, 1)
bg.inputs['Strength'].default_value = 0.22

def material(name, color, roughness=0.65):
    m = bpy.data.materials.new('IELTS_' + name)
    m.use_nodes = True
    p = next(n for n in m.node_tree.nodes if n.type == 'BSDF_PRINCIPLED')
    p.inputs['Base Color'].default_value = (*color, 1)
    p.inputs['Roughness'].default_value = roughness
    m.diffuse_color = (*color, 1)
    return m

wallmat = material('WarmPlaster', (.64,.62,.57))
floor = material('Carpet', (.19,.205,.21))
wood = material('LightOak', (.52,.36,.20))
dark = material('ChairFabric', (.12,.145,.16))
metal = material('DarkMetal', (.045,.055,.06), .3)
trim = material('PaintedTrim', (.8,.79,.74))
paper = material('Paper', (.86,.85,.79))
proxy = material('TECHNICAL_MANNEQUIN', (.40,.43,.45))
objects = []

def cube(name, loc, size, mat, bevel=0):
    bpy.ops.mesh.primitive_cube_add(size=1, location=loc)
    o=bpy.context.object
    o.name='IELTS_'+name
    o.dimensions=size
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    if mat: o.data.materials.append(mat)
    if bevel:
        mod=o.modifiers.new('Soft manufactured edges','BEVEL')
        mod.width=bevel; mod.segments=3
        o.modifiers.new('Weighted surface normals','WEIGHTED_NORMAL')
    objects.append(o)
    return o

def sphere(name, loc, size, mat):
    bpy.ops.mesh.primitive_uv_sphere_add(segments=24, ring_count=16, location=loc)
    o=bpy.context.object; o.name='IELTS_'+name; o.dimensions=size
    bpy.ops.object.transform_apply(location=False, rotation=False, scale=True)
    o.data.materials.append(mat)
    for p in o.data.polygons: p.use_smooth=True
    objects.append(o)
    return o

def cut(wall, name, loc, size):
    cutter=cube(name+'_CUTTER',loc,size,None)
    mod=wall.modifiers.new(name+'_ActualAperture','BOOLEAN')
    mod.operation='DIFFERENCE'; mod.solver='EXACT'; mod.object=cutter
    bpy.context.view_layer.objects.active=wall
    bpy.ops.object.modifier_apply(modifier=mod.name)
    wall['opening_id']=name
    objects.remove(cutter); bpy.data.objects.remove(cutter,do_unlink=True)

cube('Floor',(0,0,-.10),(4.4,5.4,.20),floor)
ceiling=cube('Ceiling',(0,0,2.9),(4.4,5.4,.20),trim)
front=cube('Wall_South',(0,-2.6,1.4),(4.4,.2,2.8),wallmat)
back=cube('Wall_North',(0,2.6,1.4),(4.4,.2,2.8),wallmat)
left=cube('Wall_West',(-2.1,0,1.4),(.2,5.0,2.8),wallmat)
right=cube('Wall_East',(2.1,0,1.4),(.2,5.0,2.8),wallmat)
cut(left,'W01',(-2.1,.35,1.65),(.5,1.8,1.3))
cut(right,'D01',(2.1,-1.55,1.05),(.5,.9,2.1))
cube('Door_Leaf',(2.12,-1.55,1.035),(.045,.86,2.07),wood,.008)
for y in [-2.035,-1.065]:
    cube('Door_Jamb',(1.98,y,1.08),(.07,.065,2.16),trim,.004)
cube('Door_Lintel',(1.98,-1.55,2.13),(.07,1.035,.075),trim,.004)
cube('Door_Handle',(1.935,-1.28,1.0),(.075,.13,.025),metal,.01)
cube('Corridor_Floor',(2.7,-1.55,-.04),(1.2,1.5,.08),floor)
cube('Corridor_Backing',(3.3,-1.55,1.2),(.12,1.5,2.4),wallmat)
for y in [-.58,1.28]:
    cube('Window_Jamb',(-1.99,y,1.65),(.12,.065,1.43),trim,.004)
for z in [.97,2.33]:
    cube('Window_Frame',(-1.99,.35,z),(.12,1.92,.065),trim,.004)
cube('Window_Sill',(-1.94,.35,.97),(.27,2.0,.055),trim,.005)
cube('Window_Mullion',(-2.015,.35,1.65),(.08,.035,1.30),trim,.004)
# A backed exterior ensures an intentional destination rather than an empty hole.
cube('Exterior_Lightwell',(-2.6,.35,1.65),(.1,2.5,2.5),trim)
for y in [-2.48,2.48]: cube('Skirting',(0,y,.055),(4,.025,.11),trim,.002)
for x in [-1.98,1.98]: cube('Skirting',(x,.45,.055),(.025,4.05,.11),trim,.002)

cube('Desk_Top',(0,0,.7125),(1.4,.75,.055),wood,.015)
for x in [-.60,.60]:
    for y in [-.285,.285]: cube('Desk_Leg',(x,y,.3425),(.055,.055,.685),metal,.007)

def chair(prefix,y,facing):
    cube(prefix+'_Seat',(0,y,.455),(.48,.46,.075),dark,.04)
    cube(prefix+'_Back',(0,y-facing*.225,.73),(.48,.07,.49),dark,.035)
    for x in [-.19,.19]:
        for dy in [-.17,.17]: cube(prefix+'_Leg',(x,y+dy,.21),(.032,.032,.42),metal,.004)
chair('ExaminerChair',.95,-1)
chair('CandidateChair',-1.05,1)
cube('Question_Card',(-.30,.13,.744),(.19,.135,.004),paper,.002)
cube('Notepad',(.30,.13,.746),(.21,.29,.008),paper,.002)
pen=cube('Pen',(.465,.12,.747),(.009,.15,.009),metal,.004)
pen.rotation_euler.z=-.16
cube('Recorder',(-.52,.02,.76),(.08,.13,.04),metal,.008)

# Deliberately neutral massing only: this is NOT the final examiner asset.
sphere('LAYOUT_PROXY_Torso',(0,.95,.86),(.43,.27,.58),proxy)
sphere('LAYOUT_PROXY_Head',(0,.94,1.32),(.19,.195,.25),proxy)
for x in [-.235,.235]:
    sphere('LAYOUT_PROXY_Arm',(x,.82,.84),(.105,.14,.37),proxy)
    sphere('LAYOUT_PROXY_Hand',(x,.46,.79),(.10,.16,.065),proxy)
for o in objects:
    if 'LAYOUT_PROXY' in o.name:
        o['asset_status']='layout_only_not_final_character'

def point(obj,target): obj.rotation_euler=(Vector(target)-obj.location).to_track_quat('-Z','Y').to_euler()
def camera(name,loc,target,lens=31.18):
    data=bpy.data.cameras.new(name); o=bpy.data.objects.new(name,data)
    scene.collection.objects.link(o); o.location=loc; data.lens=lens; data.sensor_width=36
    point(o,target); return o
main=camera('IELTS_ProductionCamera',(0,-1.12,1.18),(0,.95,1.19))
scene.camera=main
overview=camera('IELTS_OverviewCamera',(1.78,-2.3,2.38),(0,.45,.95),24)

def area(name,loc,target,power,size,color):
    d=bpy.data.lights.new(name,'AREA'); d.energy=power; d.shape='DISK'; d.size=size; d.color=color
    o=bpy.data.objects.new(name,d); scene.collection.objects.link(o); o.location=loc; point(o,target)
    return {'id':name,'location':list(loc),'target':list(target),'powerWatts':power,'sizeMeters':size}
fixtures=[area('IELTS_WindowLight',(-1.85,.35,2.0),(0,.8,1.0),170,1.5,(.84,.91,1)),
          area('IELTS_CeilingLight',(0,0,2.74),(0,.1,0),160,1.8,(1,.94,.84)),
          area('IELTS_FaceFill',(0,-1.5,1.8),(0,.95,1.2),35,1.0,(1,.95,.9))]
cube('Ceiling_Light_Panel',(0,0,2.782),(1.05,.45,.035),paper,.012)

def write(name,value): (PACK/name).write_text(json.dumps(value,ensure_ascii=False,indent=2),encoding='utf-8')
brief=json.loads((PACK/'room-brief.json').read_text(encoding='utf-8'))
brief.update(purpose='Seated face-to-face IELTS speaking practice',runtimeSurface='Windows desktop; engine selected after character validation',boundsMeters=[4,5,2.8],hero={'id':'examiner','role':'speaking examiner; proxy in layout only','reservedBoundsMeters':[.65,.75,1.55]},symmetry={'mode':'bilateral','order':2,'appliesTo':['conversation axis','table and chairs'],'exceptions':['west daylight window','east closed examination-room door']},circulation={'description':'Menu arrival directly into seated camera; access beside desk','minimumClearanceMeters':.7},productionCamera={'viewport':[1920,1080],'location':list(main.location),'target':[0,.95,1.19],'verticalFovDegrees':math.degrees(2*math.atan(math.tan(main.data.angle_x/2)*9/16))},performanceBudget={'maxSceneTriangles':500000,'maxVLowBytes':100000000,'maxHighTextureBytes':700000000},creditBudget={'maximumCredits':0,'approvalRequired':True})
brief['artifacts'].update(floorPlan='plans/floor-plan.png',reflectedCeilingPlan='plans/ceiling-plan.png',wallElevations=['plans/west-elevation.png','plans/east-elevation.png'],visualTarget='images/examiner-reference.png',openingContactSheet='masks/opening-contact-sheet.png')
write('room-brief.json',brief)
layout=json.loads((PACK/'room-layout.json').read_text(encoding='utf-8'))
layout.update(productionCamera=brief['productionCamera'],surfaces=[{'id':o.name,'dimensions':list(o.dimensions),'location':list(o.location)} for o in [front,back,left,right,ceiling]],instances=[{'id':o.name,'location':list(o.location),'dimensions':list(o.dimensions),'rotationRadians':list(o.rotation_euler),'status':o.get('asset_status','layout_geometry')} for o in objects],fixtures=fixtures)
write('room-layout.json',layout)
write('openings.json',{'schema':'game-room.opening-schedule.v1','roomId':'speaking','primaryArrival':{'openingId':'D01','exception':'Menu spawns seated candidate; ordinary 0.9m closed door appropriate to quiet exam room, no walking gameplay'},'openings':[{'id':'D01','type':'door','wall':'IELTS_Wall_East','widthMeters':.9,'heightMeters':2.1,'thresholdMeters':0,'depthMeters':.2,'destination':'backed corridor','center':[2.1,-1.55,1.05]},{'id':'W01','type':'window','wall':'IELTS_Wall_West','widthMeters':1.8,'heightMeters':1.3,'sillMeters':1.0,'depthMeters':.2,'destination':'exterior lightwell','center':[-2.1,.35,1.65]}]})
scene['stage']='Function layout, not final composition'
scene['original_scene']=original_scene
scene['character_status']='neutral massing proxy; no facial rig yet'
for screen in bpy.data.screens:
    for a in screen.areas:
        if a.type=='VIEW_3D':
            a.spaces.active.region_3d.view_perspective='CAMERA'
scene.render.filepath=str(PACK/'renders/layout-production.png')
bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'blender/speaking_layout.blend'))
print(json.dumps({'scene':scene.name,'objects':len(scene.objects),'original_scene_preserved':bpy.data.scenes.get(original_scene) is not None,'file':bpy.data.filepath,'stage':scene['stage']},ensure_ascii=False))
