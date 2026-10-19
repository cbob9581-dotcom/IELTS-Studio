"""Create the dressed examiner from CC0 source, adjusting identity before binding."""
import bpy,json
from pathlib import Path
from mpfb.services import HumanService,TargetService,AssetService,FaceService
ROOT=Path('D:/雅思')
for o in list(bpy.data.objects):
    if o.name.startswith('Examiner_'):bpy.data.objects.remove(o,do_unlink=True)
work=bpy.data.scenes.get('IELTS_Character_Work') or bpy.data.scenes.new('IELTS_Character_Work')
bpy.context.window.scene=work
macro=TargetService.get_default_macro_info_dict()
macro.update(gender=0.0,age=.60,muscle=.30,weight=.43,height=.49,proportions=.55)
macro['race']={'caucasian':1.0,'asian':0.0,'african':0.0}
body=HumanService.create_human(macro_detail_dict=macro);body.name='Examiner_Body'
identity_targets=[
    ('head/head-oval.target.gz',.55),
    ('chin/chin-width-decr.target.gz',.38),
    ('chin/chin-bones-decr.target.gz',.50),
    ('mouth/mouth-upperlip-volume-incr.target.gz',.40),
    ('mouth/mouth-lowerlip-volume-incr.target.gz',.28),
    ('eyebrows/eyebrows-angle-up.target.gz',.16),
    ('eyebrows/eyebrows-trans-up.target.gz',.08),
    ('cheek/l-cheek-volume-incr.target.gz',.16),
    ('cheek/r-cheek-volume-incr.target.gz',.16),
    ('cheek/l-cheek-bones-incr.target.gz',.10),
    ('cheek/r-cheek-bones-incr.target.gz',.10),
    ('nose/nose-width1-decr.target.gz',.14),
]
for rel,w in identity_targets:
    TargetService.load_target(body,str(ROOT/'.tools/mpfb-source/src/mpfb/data/targets'/rel),weight=w)
skin=AssetService.find_asset_absolute_path('middleage_caucasian_female.mhmat',asset_subdir='skins')
HumanService.set_character_skin(skin,body,skin_type='MAKESKIN')
rig=HumanService.add_builtin_rig(body,'default');rig.name='Examiner_Rig'
sources=[]
assets=[('clothes','toigo_female_suit_2.mhclo','Clothes'),('eyes','high-poly.mhclo','Eyes'),('eyebrows','eyebrow002.mhclo','Eyebrows'),('eyelashes','eyelashes01.mhclo','Eyelashes'),('tongue','tongue01.mhclo','Tongue'),('teeth','teeth_base.mhclo','Teeth'),('hair','elvs_short_daisy_hair.mhclo','Hair'),('clothes','shoes03.mhclo','Shoes')]
for sub,name,kind in assets:
    path=AssetService.find_asset_absolute_path(name,asset_subdir=sub)
    if not path:raise FileNotFoundError(name)
    obj=HumanService.add_mhclo_asset(path,body,asset_type='Clothes' if kind=='Shoes' else kind,subdiv_levels=1)
    obj.name='Examiner_'+kind;sources.append({'object':obj.name,'asset':path,'license':'CC-BY, Elvaerwyn' if kind=='Hair' else 'CC0'})
FaceService.load_targets(body,load_microsoft_visemes=True,load_arkit_faceunits=True)
FaceService.interpolate_targets(body)
h=bpy.data.objects['Examiner_Hair']
base=h.data.shape_keys.key_blocks[0].data if h.data.shape_keys else h.data.vertices
for key in h.data.shape_keys.key_blocks:
    for v in key.data:
        if v.co.z<1.46:v.co.z=1.46+(v.co.z-1.46)*.60
for mod in h.modifiers:
    if mod.type=='SUBSURF':mod.levels=1;mod.render_levels=2

o=bpy.data.objects['Examiner_Clothes'];mesh=o.data;adj=[[] for v in mesh.vertices]
for e in mesh.edges:a,c=e.vertices;adj[a].append(c);adj[c].append(a)
seen=set();tie=[]
for i in range(len(adj)):
    if i in seen:continue
    q=[i];seen.add(i);ids=[]
    while q:
        v=q.pop();ids.append(v)
        for n in adj[v]:
            if n not in seen:seen.add(n);q.append(n)
    if 500<len(ids)<740:
        xs=[mesh.vertices[j].co.x for j in ids]
        if max(xs)-min(xs)<.10:tie=ids;break
if not tie:raise RuntimeError('Tie component not identified; inspect clothes topology')
vg=o.vertex_groups.new(name='IELTS_Hide_Tie');vg.add(tie,1,'REPLACE')
mod=o.modifiers.new('IELTS_NoTie','MASK');mod.vertex_group=vg.name;mod.invert_vertex_group=True
(ROOT/'assets/characters/examiner/asset-sources.json').write_text(json.dumps(sources,ensure_ascii=False,indent=2),encoding='utf-8')
(ROOT/'assets/characters/examiner/identity-settings.json').write_text(json.dumps({'macro':macro,'targets':identity_targets},indent=2),encoding='utf-8')
print('DRESSED_CHARACTER_CREATED',len(rig.data.bones),'bones',len(body.data.shape_keys.key_blocks),'shape keys')
