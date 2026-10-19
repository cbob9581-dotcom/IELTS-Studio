"""Export an independent runtime tier. Run in a new Blender process, never save authoring file."""
import bpy, json, hashlib, struct
from pathlib import Path
from collections import defaultdict
ROOT=Path(__file__).resolve().parents[2]
OUT=ROOT/'rooms/speaking/runtime'
OUT.mkdir(parents=True,exist_ok=True)
scene=bpy.data.scenes['IELTS_Part2_Handoff']
bpy.context.window.scene=scene
scene.frame_set(1)
changes=[]

# Authoring DITHERED skin is effectively opaque, but glTF BLEND disables depth writes.
# Disconnect redundant alpha on solid surfaces so inner eyes/teeth cannot draw through the face.
used_materials={m for o in scene.objects if o.type=='MESH' and not o.hide_render for m in o.data.materials if m}
for material in used_materials:
    if not material.use_nodes:continue
    shader=next((n for n in material.node_tree.nodes if n.type=='BSDF_PRINCIPLED'),None)
    if shader and not any(word in material.name.lower() for word in ['eyebrow','eyelash','rebuilthair','high-poly']):
        for link in list(shader.inputs['Alpha'].links):material.node_tree.links.remove(link)
        shader.inputs['Alpha'].default_value=1.0

# Preserve the navy recolouring graph by baking its base colour into a glTF texture.
from cycles import CyclesRender
scene.render.engine=CyclesRender.bl_idname
scene.cycles.samples=1
def bake_base_color(obj,image_name):
    for other in bpy.context.view_layer.objects:other.select_set(False)
    obj.select_set(True);bpy.context.view_layer.objects.active=obj
    for material in obj.data.materials:
        nodes=material.node_tree.nodes;links=material.node_tree.links
        shader=next(n for n in nodes if n.type=='BSDF_PRINCIPLED')
        output=next(n for n in nodes if n.type=='OUTPUT_MATERIAL')
        emission=nodes.new('ShaderNodeEmission')
        source=shader.inputs['Base Color'].links[0].from_socket if shader.inputs['Base Color'].is_linked else None
        if source:links.new(source,emission.inputs['Color'])
        else:emission.inputs['Color'].default_value=shader.inputs['Base Color'].default_value
        original=output.inputs['Surface'].links[0].from_socket
        links.new(emission.outputs[0],output.inputs['Surface'])
        image=bpy.data.images.new(image_name,width=1024,height=1024,alpha=False)
        target=nodes.new('ShaderNodeTexImage');target.image=image;nodes.active=target
        bpy.ops.object.bake(type='EMIT',margin=8)
        links.new(original,output.inputs['Surface']);nodes.remove(emission)
        links.new(target.outputs['Color'],shader.inputs['Base Color'])
        if 'Clothes' in obj.name:
            for link in list(shader.inputs['Roughness'].links):links.remove(link)
            shader.inputs['Roughness'].default_value=.78
        image.pack()
bake_base_color(scene.objects['P2_Examiner_Clothes'],'IELTS_Runtime_NavySuit')
bake_base_color(scene.objects['IELTS_Desk_Top.001'],'IELTS_Runtime_LightOak')


# Bake authoring-only color adjustments, alpha fades and procedural bump into glTF textures.
# Multi-material objects bake all active material targets together, preserving each UV domain.
def bake_authored_material(obj, alpha=False, normal=False):
    from array import array
    for other in bpy.context.view_layer.objects:other.select_set(False)
    obj.select_set(True);bpy.context.view_layer.objects.active=obj
    materials=list(dict.fromkeys(m for m in obj.data.materials if m))
    entries=[]
    for mat in materials:
        nodes,links=mat.node_tree.nodes,mat.node_tree.links
        shader=next(n for n in nodes if n.type=='BSDF_PRINCIPLED')
        output=next(n for n in nodes if n.type=='OUTPUT_MATERIAL')
        original=output.inputs['Surface'].links[0].from_socket
        emission=nodes.new('ShaderNodeEmission')
        base=shader.inputs['Base Color']
        if base.is_linked:links.new(base.links[0].from_socket,emission.inputs['Color'])
        else:emission.inputs['Color'].default_value=base.default_value
        links.new(emission.outputs[0],output.inputs['Surface'])
        image=bpy.data.images.new('Runtime_'+mat.name+'_Color',width=1024,height=1024,alpha=True)
        target=nodes.new('ShaderNodeTexImage');target.image=image;nodes.active=target
        alpha_source=shader.inputs['Alpha'].links[0].from_socket if shader.inputs['Alpha'].is_linked else None
        entries.append((mat,shader,output,original,emission,image,target,alpha_source))
    bpy.ops.object.bake(type='EMIT',margin=8)
    if alpha:
        alpha_images=[]
        for mat,shader,output,original,emission,image,target,alpha_source in entries:
            for link in list(emission.inputs['Color'].links):mat.node_tree.links.remove(link)
            if alpha_source:mat.node_tree.links.new(alpha_source,emission.inputs['Color'])
            else:emission.inputs['Color'].default_value=(1,1,1,1)
            aim=bpy.data.images.new('Runtime_'+mat.name+'_Alpha',width=1024,height=1024,alpha=False,is_data=True)
            node=mat.node_tree.nodes.new('ShaderNodeTexImage');node.image=aim;mat.node_tree.nodes.active=node
            alpha_images.append(aim)
        bpy.ops.object.bake(type='EMIT',margin=8)
        for entry,aim in zip(entries,alpha_images):
            image=entry[5];base=array('f',[0])*(len(image.pixels));mask=array('f',[0])*(len(aim.pixels))
            image.pixels.foreach_get(base);aim.pixels.foreach_get(mask)
            for i in range(3,len(base),4):base[i]=max(0,min(1,mask[i-3]))
            image.pixels.foreach_set(base);image.update()
    for mat,shader,output,original,emission,image,target,alpha_source in entries:
        links=mat.node_tree.links
        links.new(original,output.inputs['Surface']);mat.node_tree.nodes.remove(emission)
        links.new(target.outputs['Color'],shader.inputs['Base Color'])
        if alpha:links.new(target.outputs['Alpha'],shader.inputs['Alpha'])
        image.pack()
    if normal:
        targets=[]
        for mat,*_ in entries:
            im=bpy.data.images.new('Runtime_'+mat.name+'_Normal',width=1024,height=1024,alpha=False,is_data=True)
            tex=mat.node_tree.nodes.new('ShaderNodeTexImage');tex.image=im;mat.node_tree.nodes.active=tex;targets.append(tex)
        bpy.ops.object.bake(type='NORMAL',normal_space='TANGENT',margin=8)
        for entry,tex in zip(entries,targets):
            mat,shader=entry[0],entry[1]
            nm=mat.node_tree.nodes.new('ShaderNodeNormalMap');mat.node_tree.links.new(tex.outputs['Color'],nm.inputs['Color'])
            mat.node_tree.links.new(nm.outputs['Normal'],shader.inputs['Normal']);tex.image.pack()
    changes.append({'object':obj.name,'baked_authoring_color':True,'baked_alpha':alpha,'baked_normal':normal})

bake_authored_material(scene.objects['P2_Examiner_Eyes'],alpha=True)
bake_authored_material(scene.objects['P2_Examiner_Eyebrows'],alpha=True)
bake_authored_material(scene.objects['P2_Examiner_Hair'],alpha=True,normal=True)

def replace_mesh(obj,keep):
    old=obj.data
    indices=sorted(keep); remap={v:i for i,v in enumerate(indices)}
    polys=[p for p in old.polygons if all(v in remap for v in p.vertices)]
    coords=[tuple(old.vertices[v].co) for v in indices]
    groups=[[(g.group,g.weight) for g in old.vertices[v].groups] for v in indices]
    group_names=[g.name for g in obj.vertex_groups]
    keys=old.shape_keys
    keydata=[(k.name,[tuple(k.data[v].co) for v in indices],k.value,k.slider_min,k.slider_max) for k in keys.key_blocks] if keys else []
    action=keys.animation_data.action if keys and keys.animation_data else None
    slot=keys.animation_data.action_slot if keys and keys.animation_data else None
    mesh=bpy.data.meshes.new(obj.name+'_Runtime');mesh.from_pydata(coords,[],[[remap[v] for v in p.vertices] for p in polys]);mesh.update()
    for material in old.materials:mesh.materials.append(material)
    for new,oldpoly in zip(mesh.polygons,polys):new.material_index=oldpoly.material_index;new.use_smooth=oldpoly.use_smooth
    for uv in old.uv_layers:
        layer=mesh.uv_layers.new(name=uv.name)
        for new,oldpoly in zip(mesh.polygons,polys):
            for newloop,oldloop in zip(new.loop_indices,oldpoly.loop_indices):layer.data[newloop].uv=uv.data[oldloop].uv
    obj.data=mesh
    obj.vertex_groups.clear()
    for name in group_names:obj.vertex_groups.new(name=name)
    for i,weights in enumerate(groups):
        for group,weight in weights:obj.vertex_groups[group].add([i],weight,'REPLACE')
    for name,points,value,minimum,maximum in keydata:
        key=obj.shape_key_add(name=name,from_mix=False)
        for p,co in zip(key.data,points):p.co=co
        key.value=value;key.slider_min=minimum;key.slider_max=maximum
    if action:
        obj.data.shape_keys.animation_data_create();obj.data.shape_keys.animation_data.action=action
        if slot:obj.data.shape_keys.animation_data.action_slot=slot
    changes.append({'object':obj.name,'vertices_before':len(old.vertices),'vertices_after':len(mesh.vertices),'morphs':len(keydata)})

for obj in list(scene.objects):
    if obj.type!='MESH' or obj.hide_render:continue
    masks=[m for m in obj.modifiers if m.type=='MASK']
    if masks:
        keep=set(range(len(obj.data.vertices)))
        for mask in masks:
            group=obj.vertex_groups.get(mask.vertex_group)
            if group is None:raise RuntimeError('Missing mask group '+mask.vertex_group)
            threshold=getattr(mask,'threshold',0.0)
            member={v.index for v in obj.data.vertices if any(g.group==group.index and g.weight>threshold for g in v.groups)}
            keep &= (set(range(len(obj.data.vertices)))-member) if mask.invert_vertex_group else member
        replace_mesh(obj,keep)
        for mask in masks:obj.modifiers.remove(mask)
    if 'HairStrands' in obj.name and False:  # Fidelity tier: preserve all authored strands.
        mesh=obj.data;parents=list(range(len(mesh.vertices)))
        def find(i):
            while parents[i]!=i:parents[i]=parents[parents[i]];i=parents[i]
            return i
        for edge in mesh.edges:
            a,b=edge.vertices;ra,rb=find(a),find(b)
            if ra!=rb:parents[rb]=ra
        components=defaultdict(list)
        for v in mesh.vertices:components[find(v.index)].append(v.index)
        keep=set(v for i,part in enumerate(sorted(components.values(),key=lambda p:min(p))) if i%4==0 for v in part)
        replace_mesh(obj,keep)
    for modifier in obj.modifiers:
        if modifier.type=='SUBSURF':modifier.show_viewport=False;modifier.show_render=False

# Preserve the author's render subdivision on eye/brow/lash meshes, including every shape key.
def preserve_facial_subdivision(obj):
    sub=next((m for m in obj.modifiers if m.type=='SUBSURF'),None)
    if not sub or sub.render_levels<1:return
    old=obj.data;keys=old.shape_keys
    values=[(k.name,[tuple(v.co) for v in k.data],k.value,k.slider_min,k.slider_max) for k in keys.key_blocks]
    action=keys.animation_data.action if keys.animation_data else None
    slot=keys.animation_data.action_slot if keys.animation_data else None
    temp=obj.copy();temp.data=old.copy();scene.collection.objects.link(temp);temp.animation_data_clear();temp.shape_key_clear()
    for modifier in list(temp.modifiers):
        if modifier.type!='SUBSURF':temp.modifiers.remove(modifier)
        else:modifier.levels=1;modifier.show_viewport=True;modifier.show_render=True
    targets=[];basis=None
    for name,points,value,minimum,maximum in values:
        for vertex,co in zip(temp.data.vertices,points):vertex.co=co
        temp.data.update();bpy.context.view_layer.update()
        graph=bpy.context.evaluated_depsgraph_get()
        evaluated=bpy.data.meshes.new_from_object(temp.evaluated_get(graph),preserve_all_data_layers=True,depsgraph=graph)
        targets.append((name,[tuple(v.co) for v in evaluated.vertices],value,minimum,maximum))
        if basis is None:basis=evaluated
        else:bpy.data.meshes.remove(evaluated)
    obj.data=basis
    for name,points,value,minimum,maximum in targets:
        key=obj.shape_key_add(name=name,from_mix=False)
        for vertex,co in zip(key.data,points):vertex.co=co
        key.value=value;key.slider_min=minimum;key.slider_max=maximum
    if action:
        obj.data.shape_keys.animation_data_create();obj.data.shape_keys.animation_data.action=action
        if slot:obj.data.shape_keys.animation_data.action_slot=slot
    bpy.data.objects.remove(temp,do_unlink=True)
    obj.modifiers.remove(sub)
    changes.append({'object':obj.name,'render_subdivision_preserved':1,'vertices_before':len(old.vertices),'vertices_after':len(basis.vertices)})
for name in ['P2_Examiner_Eyes','P2_Examiner_Eyebrows','P2_Examiner_Eyelashes']:preserve_facial_subdivision(scene.objects[name])

# Fonts and static architecture use evaluated meshes, including wall Boolean openings.
depsgraph=bpy.context.evaluated_depsgraph_get()
for obj in list(scene.objects):
    if obj.hide_render or obj.type not in {'MESH','FONT'}:continue
    if obj.type=='FONT' or (obj.type=='MESH' and not obj.data.shape_keys and not any(m.type=='ARMATURE' for m in obj.modifiers) and not obj.animation_data):
        evaluated=obj.evaluated_get(depsgraph)
        mesh=bpy.data.meshes.new_from_object(evaluated,depsgraph=depsgraph)
        if obj.type=='FONT':
            converted=bpy.data.objects.new(obj.name+'_Runtime',mesh);scene.collection.objects.link(converted)
            converted.parent=obj.parent;converted.matrix_parent_inverse=obj.matrix_parent_inverse.copy();converted.matrix_world=obj.matrix_world.copy()
            obj.hide_render=True;obj.hide_set(True)
        else:
            obj.data=mesh
            for modifier in list(obj.modifiers):obj.modifiers.remove(modifier)

for obj in scene.objects:obj.select_set(False)
exported=[]
for obj in scene.objects:
    if not obj.hide_render and 'LAYOUT_PROXY' not in obj.name and obj.type in {'MESH','ARMATURE','CAMERA','LIGHT','EMPTY'}:
        obj.hide_set(False);obj.select_set(True);exported.append(obj.name)
scene.camera=bpy.data.objects['P2_CandidateCamera']
scene.render.fps=24;scene.frame_start=1;scene.frame_end=336
props=bpy.ops.export_scene.gltf.get_rna_type().properties
from io_scene_gltf2 import get_format_items
options=dict(filepath=str(OUT/'speaking-room.glb'),export_format='GLB',use_selection=True,use_active_scene=True,export_cameras=True,export_lights=True,export_skins=True,export_morph=True,export_morph_animation=True,export_animations=True,export_force_sampling=True,export_apply=False,export_bake_animation=True,export_animation_mode='SCENE',export_anim_scene_split_object=False,export_def_bones=False,export_extras=True)
for name,value in options.items():
    if name not in props:raise RuntimeError('Exporter option unavailable: '+name)
    if props[name].type=='ENUM':
        enums={e.identifier for e in props[name].enum_items}
        if name=='export_format':enums={e[0] for e in get_format_items(None,bpy.context)}
        if value not in enums:raise RuntimeError('Exporter enum unavailable: '+name+'='+str(value))
bpy.ops.export_scene.gltf(**options)
data=(OUT/'speaking-room.glb').read_bytes();json_len=struct.unpack_from('<I',data,12)[0];document=json.loads(data[20:20+json_len])
# Explicit cutouts avoid transparent-card ordering and solid rectangle shadows.
for material in document.get('materials',[]):
    name=material.get('name','').lower()
    if 'rebuilthair' in name or 'high-poly' in name:
        material['alphaMode']='MASK';material['alphaCutoff']=.08 if 'rebuilthair' in name else .02
    elif 'eyebrow' in name or 'eyelash' in name:
        material['alphaMode']='BLEND';material.pop('alphaCutoff',None)
json_blob=json.dumps(document,separators=(',',':'),ensure_ascii=False).encode('utf-8')
json_blob+=b' '*((-len(json_blob))%4)
rest=data[20+json_len:]
data=struct.pack('<4sII',b'glTF',2,20+len(json_blob)+len(rest))+struct.pack('<I4s',len(json_blob),b'JSON')+json_blob+rest
(OUT/'speaking-room.glb').write_bytes(data)
assert document.get('skins'), 'Skeleton missing'
assert document.get('animations'), 'Animation missing'
assert len(document.get('scenes',[]))==1,'Other authoring scenes leaked into export'
morph_names={name for mesh in document.get('meshes',[]) for name in mesh.get('extras',{}).get('targetNames',[])}
assert 'jawOpen' in morph_names and 'eyeBlinkLeft' in morph_names,'Facial morphs lost'
report=dict(source='blender/speaking_room.blend',source_sha256=hashlib.sha256((ROOT/'blender/speaking_room.blend').read_bytes()).hexdigest(),scene=scene.name,output='speaking-room.glb',bytes=len(data),sha256=hashlib.sha256(data).hexdigest(),camera='P2_CandidateCamera',frames=336,fps=24,preparation_ready_seconds=12.0,changes=changes,objects=exported,skins=len(document['skins']),animations=[a.get('name','unnamed') for a in document['animations']],morph_count=len(morph_names),morph_names=sorted(morph_names),requires_instruction_audio_finished=True)
report['bone_count']=len(document['skins'][0]['joints'])
report['triangle_count']=sum(document['accessors'][p['indices']]['count']//3 for m in document['meshes'] for p in m['primitives'] if 'indices' in p)
(OUT/'manifest.json').write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf8')
print('RUNTIME_EXPORT_COMPLETE',json.dumps({k:report[k] for k in ['bytes','skins','animations','morph_count']},ensure_ascii=False))
