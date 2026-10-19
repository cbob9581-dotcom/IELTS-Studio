"""Capture the corrected seat, layered bob and subdued face animation."""
import bpy
from mathutils import Vector
from pathlib import Path
ROOT=Path('D:/雅思');s=bpy.data.scenes['IELTS_Speaking_Form'];bpy.context.window.scene=s;main=s.camera;cam=bpy.data.objects['IELTS_FormReviewCamera'];s.camera=cam;s.render.resolution_x=1120;s.render.resolution_y=800;s.render.resolution_percentage=100
for frame,label in [(1,'neutral'),(15,'speech'),(37,'blink'),(65,'listening')]:
    s.frame_set(frame);cam.location=(0,-.28,1.23);cam.rotation_euler=(Vector((0,.75,1.24))-cam.location).to_track_quat('-Z','Y').to_euler();cam.data.lens=52;s.render.filepath=str(ROOT/'rooms/speaking/renders'/('face-'+label+'-v5.png'));bpy.ops.render.render(write_still=True)
s.frame_set(1);s.camera=main;s.render.resolution_x=1600;s.render.resolution_y=900;s.render.filepath=str(ROOT/'rooms/speaking/renders/form-production-v5.png');bpy.ops.render.render(write_still=True)
s['character_stage']='Seated garment corrective, coarticulated face and layered bob v5; appearance approval pending'
bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'blender/speaking_room.blend'));print('V5_REVIEW_STILLS_SAVED')
