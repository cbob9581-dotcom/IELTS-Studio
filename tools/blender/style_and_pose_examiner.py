"""Style the MPFB examiner and integrate the seated rig into the Form scene."""
import bpy,math,json
from pathlib import Path
from mathutils import Vector,Matrix
ROOT=Path('D:/雅思')
rig=bpy.data.objects['Examiner_Rig'];body=bpy.data.objects['Examiner_Body']

def shader(obj):
    m=obj.active_material
    return m,next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED')

m,p=shader(bpy.data.objects['Examiner_Clothes']);n=m.node_tree.nodes;l=m.node_tree.links
source=next(no for no in n if no.type=='TEX_IMAGE' and no.image and no.image.name=='Fsuit2.png').outputs['Color']
hsv=n.new('ShaderNodeSeparateColor');hsv.mode=next(x.identifier for x in hsv.bl_rna.properties['mode'].enum_items if x.identifier=='HSV')
l.new(source,hsv.inputs[0])
mask=n.new('ShaderNodeMath');mask.operation='GREATER_THAN';mask.inputs[1].default_value=.14;l.new(hsv.outputs[1],mask.inputs[0])
tint=n.new('ShaderNodeMixRGB');tint.blend_type='MULTIPLY';tint.inputs[0].default_value=1;tint.inputs[2].default_value=(.025,.038,.065,1)
l.new(hsv.outputs[2],tint.inputs[1])
mix=n.new('ShaderNodeMixRGB');l.new(mask.outputs[0],mix.inputs[0]);l.new(source,mix.inputs[1]);l.new(tint.outputs[0],mix.inputs[2]);l.new(mix.outputs[0],p.inputs['Base Color'])
p.inputs['Roughness'].default_value=.72

m,p=shader(bpy.data.objects['Examiner_Hair']);n=m.node_tree.nodes;l=m.node_tree.links
source=next(no for no in n if no.type=='TEX_IMAGE' and no.image and not any(t in no.image.name.lower() for t in ['normal','bump','spec','rough'])).outputs['Color']
gray=n.new('ShaderNodeRGBToBW');l.new(source,gray.inputs[0])
ramp=n.new('ShaderNodeValToRGB');ramp.color_ramp.elements[0].position=0;ramp.color_ramp.elements[0].color=(.12,.062,.023,1)
ramp.color_ramp.elements[1].position=.45;ramp.color_ramp.elements[1].color=(.72,.49,.22,1)
l.new(gray.outputs[0],ramp.inputs[0]);l.new(ramp.outputs[0],p.inputs['Base Color']);p.inputs['Roughness'].default_value=.53

# Position and rotate bones in armature coordinates, respecting parent transforms.
rig.scale=(1.03,)*3
rig.location=(0,.75,.53-rig.data.bones['spine05'].head_local.z*1.03)
for pb in rig.pose.bones:pb.rotation_mode='QUATERNION';pb.matrix_basis=Matrix.Identity(4)
bpy.context.view_layer.update()
def aim(name,direction):
    pb=rig.pose.bones[name]
    current=(pb.tail-pb.head).normalized();target=Vector(direction).normalized()
    delta=current.rotation_difference(target)
    pb.matrix=Matrix.LocRotScale(pb.head.copy(),delta@pb.matrix.to_quaternion(),Vector((1,1,1)))
    bpy.context.view_layer.update()
for side,sign in [('L',1),('R',-1)]:
    for bone in ['upperleg01','upperleg02']:aim(bone+'.'+side,(sign*.07,-1,-.05))
    for bone in ['lowerleg01','lowerleg02']:aim(bone+'.'+side,(0,.05,-1))
    aim('foot.'+side,(0,-1,-.13))
    for bone in ['upperarm01','upperarm02']:aim(bone+'.'+side,(sign*.24,-.20,-1))
    for bone in ['lowerarm01','lowerarm02']:aim(bone+'.'+side,(-sign*.12,-1,.145))
    aim('wrist.'+side,(-sign*.12,-1,-.05))
    pb=rig.pose.bones['wrist.'+side]
    forward=Vector((-sign*.12,-1,-.05)).normalized();up=Vector((0,0,1))
    x=(-sign*(up-forward*up.dot(forward))).normalized();z=x.cross(forward).normalized()
    orientation=Matrix((x,forward,z)).transposed().to_quaternion()
    pb.matrix=Matrix.LocRotScale(pb.head.copy(),orientation,Vector((1,1,1)))
    bpy.context.view_layer.update()

form=bpy.data.scenes['IELTS_Speaking_Form']
for obj in list(bpy.data.scenes['IELTS_Character_Work'].objects):
    if obj.name not in form.objects:form.collection.objects.link(obj)
for obj in form.objects:
    if 'LAYOUT_PROXY' in obj.get('logical_id',''):
        obj.hide_render=True;obj.hide_viewport=True
    if obj.get('logical_id','').startswith('IELTS_ExaminerChair_'):
        obj.location.y=bpy.data.objects[obj['logical_id']].location.y-.20
bpy.context.window.scene=form
form['character_stage']='First dressed rig iteration; facial motion and visual QA pending'
form.render.resolution_x=1600;form.render.resolution_y=900;form.render.resolution_percentage=80
form.render.filepath=str(ROOT/'rooms/speaking/renders/examiner-first-integration.png')
# Preserve exact sitting pose for animation and future regeneration.
pose={pb.name:list(pb.rotation_quaternion) for pb in rig.pose.bones}
(ROOT/'assets/characters/examiner/seated-pose.json').write_text(json.dumps({'scale':list(rig.scale),'location':list(rig.location),'rotationQuaternions':pose},indent=2),encoding='utf-8')
bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'blender/speaking_room.blend'))
bpy.ops.render.render(write_still=True)
print('EXAMINER_FIRST_INTEGRATION',form.render.filepath)
