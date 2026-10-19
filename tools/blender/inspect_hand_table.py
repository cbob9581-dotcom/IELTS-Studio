"""Measure actual deformed hand surfaces above the tabletop across the current animation."""
import bpy,json
from pathlib import Path
ROOT=Path('D:/雅思');s=bpy.data.scenes['IELTS_Speaking_Form'];bpy.context.window.scene=s
b=bpy.data.objects['Examiner_Body'];before=s.frame_current
desk=next(o for o in s.objects if o.get('logical_id')=='IELTS_Desk_Top')
from mathutils import Vector
corners=[desk.matrix_world@Vector(v) for v in desk.bound_box]
xmin=min(p.x for p in corners);xmax=max(p.x for p in corners)
ymin=min(p.y for p in corners);ymax=max(p.y for p in corners)
top=max(p.z for p in corners)
groups={side:{g.index for g in b.vertex_groups if g.name.endswith('.'+side) and g.name.startswith(('wrist','finger'))} for side in ['L','R']}
report={'tableTop':top,'frames':[],'minimumClearanceMeters':1.0,'penetratingVertices':0}
frames=list(range(s.frame_start,s.frame_end+1))
if s.frame_end not in frames:frames.append(s.frame_end)
for f in frames:
    s.frame_set(f);dep=bpy.context.evaluated_depsgraph_get();ev=b.evaluated_get(dep);mesh=ev.to_mesh(preserve_all_data_layers=True,depsgraph=dep)
    row={'frame':f}
    for side,ids in groups.items():
        points=[ev.matrix_world@v.co for v in mesh.vertices if sum(g.weight for g in v.groups if g.group in ids)>.3]
        points=[p for p in points if xmin<p.x<xmax and ymin<p.y<ymax]
        if not points:raise RuntimeError('Hand has no sampled contact area over table')
        clearance=min(p.z for p in points)-top
        count=sum(p.z<top-.0001 for p in points)
        row[side]={'clearanceMeters':clearance,'penetratingVertices':count}
        report['minimumClearanceMeters']=min(report['minimumClearanceMeters'],clearance)
        report['penetratingVertices']+=count
    report['frames'].append(row);ev.to_mesh_clear()
s.frame_set(before)
report['passed']=report['penetratingVertices']==0 and report['minimumClearanceMeters']>=.0005
(ROOT/'assets/characters/examiner/hand-table-inspection.json').write_text(json.dumps(report,indent=2),encoding='utf-8')
print(json.dumps({k:v for k,v in report.items() if k!='frames'}))
if not report['passed']:raise RuntimeError('Hand/table clearance failed')
