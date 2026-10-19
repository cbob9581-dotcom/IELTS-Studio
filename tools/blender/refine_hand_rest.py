"""Rest open hands on the desk with relaxed, nearly parallel fingers."""
import bpy,json
from pathlib import Path
from mathutils import Vector,Matrix,Quaternion
ROOT=Path('D:/雅思');s=bpy.data.scenes['IELTS_Speaking_Form'];bpy.context.window.scene=s;s.frame_set(1)
rig=bpy.data.objects['Examiner_Rig'];data=json.loads((ROOT/'assets/characters/examiner/seated-pose.json').read_text())
for name,q in data['rotationQuaternions'].items():rig.pose.bones[name].rotation_quaternion=Quaternion(q)
bpy.context.view_layer.update()
def aim(name,direction):
    p=rig.pose.bones[name];delta=(p.tail-p.head).normalized().rotation_difference(Vector(direction).normalized())
    p.matrix=Matrix.LocRotScale(p.head.copy(),delta@p.matrix.to_quaternion(),Vector((1,1,1)));bpy.context.view_layer.update()
for side,sign in [('L',1),('R',-1)]:
    for bone in ['lowerarm01','lowerarm02']:aim(bone+'.'+side,(-sign*.12,-1,.050))
    p=rig.pose.bones['wrist.'+side];f=Vector((-sign*.12,-1,0)).normalized();up=Vector((0,0,1));x=(-sign*(up-f*up.dot(f))).normalized()
    p.matrix=Matrix.LocRotScale(p.head.copy(),Matrix((x,f,x.cross(f))).transposed().to_quaternion(),Vector((1,1,1)));bpy.context.view_layer.update()
    for digit in range(2,6):
        for part in range(1,4):aim(f'finger{digit}-{part}.{side}',(-sign*.10,-1,-.025 if part==2 else .005))
    for part in range(1,4):aim(f'finger1-{part}.{side}',(-sign*.55,-.85,-.07))
pose={p.name:list(p.rotation_quaternion) for p in rig.pose.bones}
data['rotationQuaternions']=pose;data['handRest']='Relaxed open hands, flatter finger articulation'
(ROOT/'assets/characters/examiner/seated-pose.json').write_text(json.dumps(data,indent=2),encoding='utf-8')
print('RELAXED_HAND_REST_POSE')
