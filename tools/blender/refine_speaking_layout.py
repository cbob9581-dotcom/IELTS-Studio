"""Small layout revision; run after build_speaking_layout.py, before capture_layout_review.py."""
import bpy
from mathutils import Vector
s=bpy.context.scene
s.render.use_compositing=False
s.camera=bpy.data.objects['IELTS_ProductionCamera']
s.camera.rotation_euler=(Vector((0,.95,1.08))-s.camera.location).to_track_quat('-Z','Y').to_euler()
t=bpy.data.objects['IELTS_LAYOUT_PROXY_Torso'];t.dimensions=(.40,.25,.47);t.location.z=.85
def ellipsoid(name,loc,size):
    o=bpy.data.objects.get(name)
    if not o:
        bpy.ops.mesh.primitive_uv_sphere_add(segments=24,ring_count=16,location=loc)
        o=bpy.context.object;o.name=name;o.data.materials.append(t.active_material)
        for p in o.data.polygons:p.use_smooth=True
    o.location=loc;o.dimensions=size;o['asset_status']='layout_only_not_final_character'
ellipsoid('IELTS_LAYOUT_PROXY_Neck',(0,.95,1.08),(.105,.105,.13))
bpy.data.objects['IELTS_LAYOUT_PROXY_Head'].location.z=1.20
for i,x in enumerate([-.22,.22]):
    ellipsoid('IELTS_LAYOUT_PROXY_Forearm_'+str(i),(x,.60,.775),(.09,.36,.085))
    ellipsoid('IELTS_LAYOUT_PROXY_Thigh_'+str(i),(x*.48,.76,.54),(.16,.40,.14))
    ellipsoid('IELTS_LAYOUT_PROXY_Shin_'+str(i),(x*.48,.60,.28),(.12,.14,.43))
    ellipsoid('IELTS_LAYOUT_PROXY_Shoe_'+str(i),(x*.48,.54,.07),(.15,.25,.13))
s.render.resolution_percentage=100
s.render.filepath='D:/雅思/rooms/speaking/renders/layout-production.png'
bpy.ops.render.render(write_still=True)
