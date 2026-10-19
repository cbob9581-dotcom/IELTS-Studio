"""Pose-specific seated garment corrective, calculated through inverse skinning."""
import bpy,json
from pathlib import Path
from mathutils import Matrix,Vector
ROOT=Path('D:/雅思');s=bpy.data.scenes['IELTS_Speaking_Form'];bpy.context.window.scene=s;s.frame_set(1)
o=bpy.data.objects['Examiner_Clothes'];rig=bpy.data.objects['Examiner_Rig']
seat=next(x for x in s.objects if x.get('logical_id')=='IELTS_ExaminerChair_Seat')
ps=[seat.matrix_world@Vector(v) for v in seat.bound_box];top=max(p.z for p in ps);xmin=min(p.x for p in ps);xmax=max(p.x for p in ps);ymin=min(p.y for p in ps);ymax=max(p.y for p in ps)
key=o.data.shape_keys.key_blocks.get('IELTS_SeatedGarmentContact') if o.data.shape_keys else None
if key:o.shape_key_remove(key)
key=o.shape_key_add(name='IELTS_SeatedGarmentContact');key.value=1
states=[(m,m.show_viewport) for m in o.modifiers if m.type=='MASK']
for m,_ in states:m.show_viewport=False
worldinv=o.matrix_world.inverted().to_3x3();count=0;maxmove=0
for attempt in range(2):
    bpy.context.view_layer.update();ev=o.evaluated_get(bpy.context.evaluated_depsgraph_get());me=ev.to_mesh();positions=[ev.matrix_world@v.co for v in me.vertices];ev.to_mesh_clear()
    assert len(positions)==len(o.data.vertices),'Corrective needs unmasked source topology'
    for i,p in enumerate(positions):
        if not (xmin-.025<p.x<xmax+.025 and ymin-.025<p.y<ymax+.015 and .22<p.z<top+.002):continue
        displacement=top+.002-p.z
        if displacement<=0:continue
        v=o.data.vertices[i];matrix=Matrix(((0,0,0,0),)*4);total=0
        for g in v.groups:
            name=o.vertex_groups[g.group].name;pb=rig.pose.bones.get(name)
            if pb and pb.bone.use_deform:
                matrix+=g.weight*(pb.matrix@pb.bone.matrix_local.inverted());total+=g.weight
        if total<.01:raise RuntimeError('Unweighted garment contact vertex')
        matrix=matrix*(1/total);local_delta=matrix.to_3x3().inverted()@(worldinv@Vector((0,0,displacement)))
        key.data[i].co+=local_delta;count+=1;maxmove=max(maxmove,displacement)
for m,enabled in states:m.show_viewport=enabled
bpy.context.view_layer.update()
info={'method':'pose-specific inverse-skinning garment contact corrective','seatTopMeters':top,'contactGapMeters':.002,'correctedVertexOperations':count,'maximumOriginalPenetrationMeters':maxmove,'scope':'seated examiner pose, not general cloth physics'}
(ROOT/'assets/characters/examiner/seated-contact-correction.json').write_text(json.dumps(info,indent=2),encoding='utf-8');print('SEATED_GARMENT_CORRECTIVE',json.dumps(info))
