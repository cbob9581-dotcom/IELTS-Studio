"""Evaluate handoff contacts, table clearance and attachment/release over all frames."""
import bpy,json
from pathlib import Path
from mathutils import Vector
ROOT=Path('D:/雅思');s=bpy.data.scenes['IELTS_Part2_Handoff'];bpy.context.window.scene=s;before=s.frame_current
desk=next(o for o in s.objects if o.get('logical_id')=='IELTS_Desk_Top');ps=[desk.matrix_world@Vector(v) for v in desk.bound_box];lo=[min(p[i] for p in ps) for i in range(3)];hi=[max(p[i] for p in ps) for i in range(3)]
body=bpy.data.objects['P2_Examiner_Body'];clothes=bpy.data.objects['P2_Examiner_Clothes'];groups={g.index for g in body.vertex_groups if g.name.startswith(('wrist','finger','lowerarm'))}
props=[('card','P2_TopicCard',45,91),('paper','P2_BlankPaper',137,179),('pen','P2_Pen',219,259)]
report={'frames':336,'tableTopMeters':hi[2],'handBelowTableVertices':0,'garmentTableIntersectionVertices':0,'minimumHandClearanceMeters':1,'minimumPropClearanceMeters':1,'maxAttachmentErrorMeters':0,'maxReleasedDriftMeters':0,'restingProps':{},'badFrames':[]};released={}
for f in range(1,337):
    s.frame_set(f);dep=bpy.context.evaluated_depsgraph_get();bad={}
    for o in [body,clothes]:
        ev=o.evaluated_get(dep);me=ev.to_mesh(preserve_all_data_layers=True,depsgraph=dep);points=[ev.matrix_world@v.co for v in me.vertices if o==clothes or sum(g.weight for g in v.groups if g.group in groups)>.35];points=[p for p in points if lo[0]<p.x<hi[0] and lo[1]<p.y<hi[1]]
        if o==body:
            if points:report['minimumHandClearanceMeters']=min(report['minimumHandClearanceMeters'],min(p.z-hi[2] for p in points))
            count=sum(p.z<hi[2]-.0002 for p in points);report['handBelowTableVertices']+=count
        else:
            count=sum(lo[2]<p.z<hi[2]-.0002 for p in points);report['garmentTableIntersectionVertices']+=count
        if count:bad[o.name]=count
        ev.to_mesh_clear()
    for label,name,pick,release in props:
        o=bpy.data.objects[name];matrix=o.evaluated_get(dep).matrix_world;pts=[matrix@Vector(v) for v in o.bound_box];clearance=min(p.z for p in pts)-hi[2];report['minimumPropClearanceMeters']=min(report['minimumPropClearanceMeters'],clearance)
        if pick<=f<release:
            anchor=bpy.data.objects['P2_Grip_'+label].evaluated_get(dep);error=(matrix.translation-anchor.matrix_world.translation).length;report['maxAttachmentErrorMeters']=max(report['maxAttachmentErrorMeters'],error)
        if f==release+6:released[label]=matrix.copy();report['restingProps'][label]={'bottomClearanceMeters':clearance,'center':list(matrix.translation)}
        if f>release+6:report['maxReleasedDriftMeters']=max(report['maxReleasedDriftMeters'],(matrix.translation-released[label].translation).length)
    if bad:report['badFrames'].append({'frame':f,**bad})
s.frame_set(before);report['passed']=report['handBelowTableVertices']==0 and report['garmentTableIntersectionVertices']==0 and report['minimumPropClearanceMeters']>=-.0001 and report['maxAttachmentErrorMeters']<1e-5 and report['maxReleasedDriftMeters']<1e-5 and all(0<=x['bottomClearanceMeters']<.004 for x in report['restingProps'].values())
(ROOT/'assets/characters/examiner/part2-handoff-inspection.json').write_text(json.dumps(report,indent=2),encoding='utf-8');print('PART2_INSPECTION',json.dumps({k:v for k,v in report.items() if k!='badFrames'}));print('BAD_FRAMES',report['badFrames'][:15])
