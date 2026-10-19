"""Read source, render one matching-camera reference, and record actual authored lights. Never save .blend."""
import bpy,json,math
from pathlib import Path
from mathutils import Quaternion
root=Path(__file__).resolve().parents[2]
s=bpy.data.scenes['IELTS_Part2_Handoff'];bpy.context.window.scene=s;s.frame_set(1)
camera=s.objects['P2_CandidateCamera'];s.camera=camera
camera.data.sensor_fit='VERTICAL';camera.data.sensor_height=2*camera.data.lens*math.tan(.5207952913920271/2)
s.render.resolution_x=1152;s.render.resolution_y=800;s.render.resolution_percentage=100
s.render.use_compositing=False
s.render.image_settings.media_type='IMAGE';s.render.image_settings.file_format='PNG'
s.render.filepath=str(root/'docs/previews/blender-material-reference.png')
fixtures=[]
convert=Quaternion((1,0,0),-math.pi/2)
for o in s.objects:
 if o.type!='LIGHT' or o.hide_render:continue
 q=convert@o.matrix_world.to_quaternion()
 fixtures.append({'id':o.name,'location':list(o.matrix_world.translation),'rotation':[q.x,q.y,q.z,q.w],
  'powerWatts':o.data.energy,'color':list(o.data.color),'sizeMeters':getattr(o.data,'size',1)})
world=next(n for n in s.world.node_tree.nodes if n.type=='BACKGROUND')
(root/'rooms/speaking/fixture-manifest.json').write_text(json.dumps({'roomId':'speaking','viewTransform':s.view_settings.view_transform,'exposure':s.view_settings.exposure,
 'worldColor':list(world.inputs['Color'].default_value)[:3],'worldStrength':world.inputs['Strength'].default_value,'fixtures':fixtures},indent=2),encoding='utf-8')
print('REFERENCE_ENGINE',s.render.engine)
bpy.ops.render.render(write_still=True)
print('REFERENCE_COMPLETE',s.render.filepath)
