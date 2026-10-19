"""Check actual evaluated hair skinning against the head-bone transform."""
import bpy,json,math
from pathlib import Path
from mathutils import Quaternion
ROOT=Path('D:/雅思');s=bpy.data.scenes['IELTS_Speaking_Form'];bpy.context.window.scene=s
rig=bpy.data.objects['Examiner_Rig'];before=s.frame_current;report={'frames':[],'maximumBindingErrorMeters':0,'fullHeadWeights':True}
objects=[bpy.data.objects[n] for n in ['Examiner_Hair','Examiner_HairStrands','Examiner_HairRootCap'] if n in bpy.data.objects]
for o in objects:
    ids={g.index for g in o.vertex_groups if g.name=='head'}
    assert o.parent==rig
    if any(abs(sum(g.weight for g in v.groups if g.group in ids)-1)>1e-6 for v in o.data.vertices):report['fullHeadWeights']=False
def inspect(label):
    bpy.context.view_layer.update();dep=bpy.context.evaluated_depsgraph_get();row={'pose':label,'errors':{}}
    transform=rig.matrix_world@rig.pose.bones['head'].matrix@rig.data.bones['head'].matrix_local.inverted()
    for o in objects:
        ev=o.evaluated_get(dep);me=ev.to_mesh();maximum=0
        for i in range(0,len(o.data.vertices),max(1,len(o.data.vertices)//500)):
            error=((transform@o.data.vertices[i].co)-(ev.matrix_world@me.vertices[i].co)).length;maximum=max(maximum,error)
        row['errors'][o.name]=maximum;report['maximumBindingErrorMeters']=max(report['maximumBindingErrorMeters'],maximum);ev.to_mesh_clear()
    report['frames'].append(row)
for f in [1,15,46,110,147]:s.frame_set(f);inspect('frame '+str(f))
s.frame_set(1);pb=rig.pose.bones['head'];original=pb.rotation_quaternion.copy()
for yaw in [-20,20]:pb.rotation_quaternion=original@Quaternion((0,1,0),math.radians(yaw));inspect('head turn '+str(yaw))
pb.rotation_quaternion=original;s.frame_set(before);bpy.context.view_layer.update()
report['passed']=report['fullHeadWeights'] and report['maximumBindingErrorMeters']<1e-5
(ROOT/'assets/characters/examiner/hair-binding-inspection.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
print('HAIR_BINDING',json.dumps(report))
if not report['passed']:raise RuntimeError('Hair does not follow the head correctly')
