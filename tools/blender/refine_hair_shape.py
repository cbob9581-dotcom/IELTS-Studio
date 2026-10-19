"""Correct the cap silhouette and stepped strips found in the eight-view review."""
import bpy,json
from pathlib import Path
from mathutils import Vector
from mathutils.bvhtree import BVHTree
ROOT=Path('D:/雅思');h=bpy.data.objects['Examiner_Hair']
if not h.get('silhouette_refined_v3'):
    for key in h.data.shape_keys.key_blocks:
        for v in key.data:
            # Preserve the scalp-fitting roots; taper only the lower curtain.
            t=max(0,min(1,(1.51-v.co.z)/.16))
            v.co.x*=1-.09*t
            v.co.y=-.032+(v.co.y+.032)*(1-.05*t)
    h['silhouette_refined_v3']=True
sub=next((x for x in h.modifiers if x.type=='SUBSURF'),None) or h.modifiers.new('Smooth hair strips','SUBSURF')
sub.levels=1;sub.render_levels=2
for p in h.data.polygons:p.use_smooth=True
# Keep smoothed roots outside the actual deformed scalp, rather than shrinking
# the entire cap through the head. Hide only pose and masking during this query.
b=bpy.data.objects['Examiner_Body'];states=[(m,m.show_viewport) for m in b.modifiers if m.type in {'ARMATURE','MASK'}]
for mod,_ in states:mod.show_viewport=False
bpy.context.view_layer.update();ev=b.evaluated_get(bpy.context.evaluated_depsgraph_get());me=ev.to_mesh()
tree=BVHTree.FromPolygons([v.co.copy() for v in me.vertices],[list(p.vertices) for p in me.polygons]);ev.to_mesh_clear()
for mod,enabled in states:mod.show_viewport=enabled
center=Vector((0,-.025,1.49))
for key in h.data.shape_keys.key_blocks:
    for v in key.data:
        if v.co.z<1.48:continue
        direction=(v.co-center).normalized();hit,normal,index,distance=tree.ray_cast(center,direction,.3)
        if hit is not None and (hit-center).length+.008>(v.co-center).length:v.co=hit+direction*.008
bpy.context.view_layer.update()
m=h.active_material;p=next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED')
for n in m.node_tree.nodes:
    if n.type=='VALTORGB':
        n.color_ramp.elements[0].position=0;n.color_ramp.elements[0].color=(.065,.028,.009,1)
        n.color_ramp.elements[1].position=.90;n.color_ramp.elements[1].color=(.58,.35,.12,1)
p.inputs['Roughness'].default_value=.46
if 'Anisotropic IOR Level' in p.inputs:p.inputs['Anisotropic IOR Level'].default_value=.65
print('HAIR_SILHOUETTE_AND_SURFACE_REFINED')
