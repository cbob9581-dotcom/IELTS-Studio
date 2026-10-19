"""Capture real character deformation and architectural Form review evidence."""
import bpy,json,math
from pathlib import Path
from mathutils import Vector
ROOT=Path('D:/雅思');PACK=ROOT/'rooms/speaking';s=bpy.data.scenes['IELTS_Speaking_Form'];bpy.context.window.scene=s
main=s.camera;frame=s.frame_current
def cam(name,loc,target,lens):
    o=bpy.data.objects.get(name)
    if not o:
        d=bpy.data.cameras.new(name);o=bpy.data.objects.new(name,d);s.collection.objects.link(o)
    o.location=loc;o.rotation_euler=(Vector(target)-o.location).to_track_quat('-Z','Y').to_euler();o.data.lens=lens
    return o
def render(name,loc=None,target=None,lens=16):
    if loc:s.camera=cam('IELTS_FormReviewCamera',loc,target,lens)
    s.render.filepath=str(PACK/'renders'/name);bpy.ops.render.render(write_still=True)
s.render.resolution_x=960;s.render.resolution_y=600;s.render.resolution_percentage=100
s.frame_set(1)
corners=[]
for i,(x,y) in enumerate([(-1.80,-2.28),(1.80,-2.28),(1.80,2.28),(-1.80,2.28)],1):
    name=f'form-corner-{i}.png';render(name,(x,y,1.50),(0,0,1.25),10.4);corners.append('renders/'+name)
ceilings=[]
for name,loc,target in [('form-ceiling-up.png',(0,0,1.2),(0,0,2.8)),('form-ceiling-oblique-a.png',(-1.7,-2,1.3),(.2,.3,2.72)),('form-ceiling-oblique-b.png',(1.7,2,1.3),(-.2,-.3,2.72))]:
    render(name,loc,target,14);ceilings.append('renders/'+name)
# Facial views are evidence of actual animated mesh deformation.
for number,label in [(1,'neutral'),(15,'vowel'),(46,'blink')]:
    s.frame_set(number);render('face-'+label+'.png',(0,-.28,1.23),(0,.75,1.24),52)
s.camera=main;s.frame_set(1)
s.render.resolution_x=1600;s.render.resolution_y=900
render('form-production.png')
s.render.filepath=str(PACK/'renders/form-production.png')
for screen in bpy.data.screens:
    for a in screen.areas:
        if a.type=='VIEW_3D':a.spaces.active.region_3d.view_perspective='CAMERA'
report=json.loads((PACK/'final-report.json').read_text(encoding='utf-8'))
report['reviewRenders']={'corners':corners,'ceiling':ceilings,'production':'renders/form-production.png','face':['renders/face-neutral.png','renders/face-vowel.png','renders/face-blink.png']}
report['notes']=['First rigged examiner and environment review. Runtime engine and cloud APIs not connected.','Authoring camera calibrated to 50 degree horizontal FOV for face readability.','Form and runtime remain unapproved until user reviews the actual result.']
(PACK/'final-report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'blender/speaking_room.blend'))
print('FORM_REVIEW_CAPTURES_SAVED',len(corners)+len(ceilings)+4)
