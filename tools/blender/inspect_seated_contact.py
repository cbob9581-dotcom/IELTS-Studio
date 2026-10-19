"""Check evaluated garment vertices against the full examiner seat and back."""
import bpy,json
from pathlib import Path
from mathutils import Vector
from mathutils.bvhtree import BVHTree
ROOT=Path('D:/雅思');s=bpy.data.scenes['IELTS_Speaking_Form'];bpy.context.window.scene=s;before=s.frame_current
fixtures=[o for o in s.objects if o.get('logical_id') in ['IELTS_ExaminerChair_Seat','IELTS_ExaminerChair_Back']]
bounds={o.name:([min((o.matrix_world@Vector(v))[i] for v in o.bound_box) for i in range(3)],[max((o.matrix_world@Vector(v))[i] for v in o.bound_box) for i in range(3)]) for o in fixtures}
o=bpy.data.objects['Examiner_Clothes'];report={'frames':[],'penetratingVertices':0,'bvhSurfaceOverlaps':[],'minimumSeatClearanceMeters':1}
for f in range(s.frame_start,s.frame_end+1):
    s.frame_set(f);dep=bpy.context.evaluated_depsgraph_get();ev=o.evaluated_get(dep);me=ev.to_mesh();positions=[ev.matrix_world@v.co for v in me.vertices];row={'frame':f,'inside':{}}
    for fixture in fixtures:
        lo,hi=bounds[fixture.name];count=sum(all(lo[i]+.0001<p[i]<hi[i]-.0001 for i in range(3)) for p in positions);row['inside'][fixture.name]=count;report['penetratingVertices']+=count
        if fixture.get('logical_id')=='IELTS_ExaminerChair_Seat':
            contact=[p.z-hi[2] for p in positions if lo[0]<p.x<hi[0] and lo[1]<p.y<hi[1] and p.z>.22]
            report['minimumSeatClearanceMeters']=min(report['minimumSeatClearanceMeters'],min(contact))
        if f in [1,46,110,147]:
            a=BVHTree.FromPolygons(positions,[list(p.vertices) for p in me.polygons]);fe=fixture.evaluated_get(dep);fm=fe.to_mesh();b=BVHTree.FromPolygons([fe.matrix_world@v.co for v in fm.vertices],[list(p.vertices) for p in fm.polygons]);overlaps=len(a.overlap(b));fe.to_mesh_clear();report['bvhSurfaceOverlaps'].append({'frame':f,'fixture':fixture.name,'trianglePairs':overlaps})
    ev.to_mesh_clear();report['frames'].append(row)
s.frame_set(before);report['passed']=report['penetratingVertices']==0 and all(x['trianglePairs']==0 for x in report['bvhSurfaceOverlaps'])
(ROOT/'assets/characters/examiner/seat-contact-inspection.json').write_text(json.dumps(report,indent=2),encoding='utf-8');print('SEAT_CONTACT',json.dumps({k:v for k,v in report.items() if k!='frames'}))
if not report['passed']:raise RuntimeError('Seat/clothes collision remains')
