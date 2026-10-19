"""Actual candidate camera and close grasp views of the independent handoff clip."""
import bpy
from pathlib import Path
from mathutils import Vector
ROOT=Path('D:/雅思');s=bpy.data.scenes['IELTS_Part2_Handoff'];bpy.context.window.scene=s;main=s.camera
s.render.resolution_x=1120;s.render.resolution_y=800;s.render.resolution_percentage=100
for f in [1,45,63,89,153,235,336]:
    s.frame_set(f);s.render.filepath=str(ROOT/'rooms/speaking/renders'/f'part2-stage-{f}.png');bpy.ops.render.render(write_still=True)
cam=bpy.data.objects.get('P2_ReviewCamera')
if not cam:
    d=bpy.data.cameras.new('P2_ReviewCamera');cam=bpy.data.objects.new('P2_ReviewCamera',d);s.collection.objects.link(cam)
s.camera=cam
for f,label,loc,target in [(63,'card',(.20,.08,1.02),(-.22,.25,.87)),(153,'paper',(-.20,.06,1.02),(.23,.25,.87)),(235,'pen',(-.08,.08,1.02),(.35,.22,.87))]:
    s.frame_set(f);cam.location=loc;cam.rotation_euler=(Vector(target)-cam.location).to_track_quat('-Z','Y').to_euler();cam.data.lens=60;s.render.filepath=str(ROOT/'rooms/speaking/renders'/f'part2-grip-{label}.png');bpy.ops.render.render(write_still=True)
s.camera=main;s.frame_set(336);s.render.resolution_x=1280;s.render.resolution_y=720
bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'blender/speaking_room.blend'));print('PART2_REAL_REVIEW_CAPTURES_SAVED')
