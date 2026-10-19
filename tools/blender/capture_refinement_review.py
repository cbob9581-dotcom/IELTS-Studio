"""Real Blender renders for the user's facial-detail and table-contact corrections."""
import bpy,json
from pathlib import Path
from mathutils import Vector
ROOT=Path('D:/雅思');s=bpy.data.scenes['IELTS_Speaking_Form'];bpy.context.window.scene=s
main=s.camera;cam=bpy.data.objects['IELTS_FormReviewCamera'];s.frame_set(1)
def capture(name,loc=None,target=None,lens=52):
    if loc:
        s.camera=cam;cam.location=loc;cam.rotation_euler=(Vector(target)-cam.location).to_track_quat('-Z','Y').to_euler();cam.data.lens=lens
    s.render.filepath=str(ROOT/'rooms/speaking/renders'/name);bpy.ops.render.render(write_still=True)
s.render.resolution_x=1120;s.render.resolution_y=800;s.render.resolution_percentage=100
for f,label in [(1,'neutral'),(15,'vowel'),(46,'blink')]:
    s.frame_set(f);capture('face-'+label+'-v2.png',(0,-.28,1.23),(0,.75,1.24))
s.frame_set(1)
capture('hands-table-v2.png',(.55,-.48,1.02),(0,.28,.77),52)
capture('hands-table-side-v2.png',(.72,.1,.82),(0,.31,.76),58)
s.camera=main;s.render.resolution_x=1600;s.render.resolution_y=900
capture('form-production-v2.png')
s['character_stage']='Appearance and hand-contact correction v2; awaiting user visual review'
bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'blender/speaking_room.blend'))
print('REFINEMENT_V2_CAPTURES_SAVED')
