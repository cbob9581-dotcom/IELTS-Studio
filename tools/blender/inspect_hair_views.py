"""Multi-angle close renders of the actual examiner hair."""
import bpy,math
from pathlib import Path
from mathutils import Vector
ROOT=Path('D:/雅思');s=bpy.data.scenes['IELTS_Speaking_Form'];bpy.context.window.scene=s;s.frame_set(1)
main=s.camera;cam=bpy.data.objects['IELTS_FormReviewCamera'];s.camera=cam
s.render.resolution_x=800;s.render.resolution_y=800;s.render.resolution_percentage=100
center=Vector((0,.75,1.22))
views=[('front',0,0),('front-left',-45,0),('left',-90,0),('back-left',-135,0),('back',180,0),('right',90,0),('front-right',45,0),('top',0,70)]
for name,az,el in views:
    a=math.radians(az);e=math.radians(el);cam.location=center+Vector((math.sin(a)*math.cos(e),-math.cos(a)*math.cos(e),math.sin(e)))*1.05
    cam.rotation_euler=(center-cam.location).to_track_quat('-Z','Y').to_euler();cam.data.lens=58
    s.render.filepath=str(ROOT/'rooms/speaking/renders'/('hair-check-'+name+'.png'));bpy.ops.render.render(write_still=True)
s.camera=main;s.render.resolution_x=1600;s.render.resolution_y=900
print('HAIR_EIGHT_VIEWS_RENDERED')
