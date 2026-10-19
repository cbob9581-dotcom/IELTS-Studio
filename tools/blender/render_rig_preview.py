"""Background-only review movie. Does not change the saved authoring camera."""
import bpy
from pathlib import Path
s=bpy.data.scenes['IELTS_Speaking_Form'];bpy.context.window.scene=s
s.render.resolution_x=768;s.render.resolution_y=432;s.render.resolution_percentage=100
s.camera.data.lens=65
s.render.use_compositing=False
def option(obj,key,value):
    valid=[e.identifier for e in obj.bl_rna.properties[key].enum_items]
    if value not in valid:raise RuntimeError(str((key,value,valid)))
    setattr(obj,key,value)
option(s.render.image_settings,'media_type','VIDEO')
option(s.render.ffmpeg,'format','MPEG4')
option(s.render.ffmpeg,'codec','H264')
option(s.render.ffmpeg,'audio_codec','AAC')
option(s.render.ffmpeg,'constant_rate_factor','MEDIUM')
if hasattr(s,'eevee') and hasattr(s.eevee,'taa_render_samples'):s.eevee.taa_render_samples=24
s.render.filepath='D:/雅思/rooms/speaking/renders/rig-speech-preview-v6.mp4'
bpy.ops.render.render(animation=True)
print('RIG_MOVIE_RENDERED',s.render.filepath)
