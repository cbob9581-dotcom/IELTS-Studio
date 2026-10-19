"""Render the real handoff scene as a silent 24 FPS review movie."""
import bpy
s=bpy.data.scenes['IELTS_Part2_Handoff'];bpy.context.window.scene=s
s.render.resolution_x=768;s.render.resolution_y=432;s.render.resolution_percentage=100;s.render.fps=24;s.render.use_compositing=False
def option(obj,key,value):
    valid=[e.identifier for e in obj.bl_rna.properties[key].enum_items]
    if value not in valid:raise RuntimeError(str((key,value,valid)))
    setattr(obj,key,value)
option(s.render.image_settings,'media_type','VIDEO');option(s.render.ffmpeg,'format','MPEG4');option(s.render.ffmpeg,'codec','H264');option(s.render.ffmpeg,'constant_rate_factor','MEDIUM')
s.render.filepath='D:/雅思/rooms/speaking/renders/part2-handoff-preview.mp4'
if hasattr(s,'eevee'):s.eevee.taa_render_samples=24
bpy.ops.render.render(animation=True);print('PART2_MOVIE_RENDERED',s.render.filepath)
