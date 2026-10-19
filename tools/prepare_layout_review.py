"""Prepare the review package without asserting user approval or final character readiness."""
from pathlib import Path
import json
import sys
from PIL import Image,ImageDraw,ImageOps
sys.stdout.reconfigure(encoding='utf-8')

ROOT=Path(__file__).resolve().parents[1]
PACK=ROOT/'rooms/speaking'
review_file=PACK/'milestone-reviews.json'
if review_file.exists() and json.loads(review_file.read_text(encoding='utf-8')).get('function',{}).get('status')=='approved':
    raise RuntimeError('Approved Function package is locked. Create a reviewed revision before regenerating references or approvals.')
def load(name):return json.loads((PACK/name).read_text(encoding='utf-8'))
def save(name,obj):(PACK/name).write_text(json.dumps(obj,ensure_ascii=False,indent=2),encoding='utf-8')
for name in ['speaking','listening','reading','writing']:(ROOT/'app/modules'/name).mkdir(parents=True,exist_ok=True)
for name in ['asr','dialogue','tts']:(ROOT/'services'/name).mkdir(parents=True,exist_ok=True)
(ROOT/'assets/characters/examiner').mkdir(parents=True,exist_ok=True)

# Normalize the tonal range of engineering captures; geometry stays mesh-derived.
for name in ['floor-plan','ceiling-plan','west-elevation','east-elevation']:
    p=PACK/'plans'/f'{name}.png'
    raw=p.with_name(name+'-mesh-render.png')
    if not raw.exists():Image.open(p).save(raw)
    img=Image.open(raw).convert('L')
    ImageOps.autocontrast(img,cutoff=0.3).save(p)

# Masks are wall elevations in the actual wall coordinate system.
for kind,rect in [('D01',(310,100,489,519)),('W01',(290,54,649,313))]:
    img=Image.new('L',(1000,560),0)
    ImageDraw.Draw(img).rectangle(rect,fill=255)
    if kind=='D01':
        # 2.1m-high aperture must touch floor (bottom); width is 0.9 of a 5m wall.
        img=Image.new('L',(1000,560),0);ImageDraw.Draw(img).rectangle((100,140,279,559),fill=255)
    else:
        img=Image.new('L',(1000,560),0);ImageDraw.Draw(img).rectangle((390,100,749,359),fill=255)
    img.save(PACK/'masks'/f'{kind}.png')

brief=load('room-brief.json')
brief['productionCamera']['target']=[0,.95,1.08]
brief['symmetry']['appliesTo']=['architecture','barriers','circulation','repeatedProps']
save('room-brief.json',brief)
layout=load('room-layout.json')
geometry=layout.get('geometryObjects',layout['instances'])
layout['geometryObjects']=geometry
groups=[('desk','IELTS_Desk_',1,[1.4,.75,.74],'negative-y','floor','images/desk-reference.png'),
        ('chair','IELTS_ExaminerChair_',2,[.48,.49,.975],'negative-y','floor','images/chair-reference.png'),
        ('stationery','stationery',1,[1.08,.35,.04],'negative-y','freestanding','renders/layout-production.png'),
        ('examiner-envelope','IELTS_LAYOUT_PROXY_',1,[.65,.75,1.55],'negative-y','floor','images/examiner-reference.png'),
        ('ceiling-fixture','IELTS_Ceiling_Light_Panel',1,[1.05,.45,.035],'negative-z','ceiling','plans/ceiling-plan.png')]
props=load('props.json')
props['generation'].update(provider='none-procedural',apiModel=None,pricingVerifiedAt=None,creditsPerTexturedTask=0,hardCreditCeiling=0,notes='No Meshy submissions: zero paid budget; furniture is authored procedurally. Stock validator assumes a positive Meshy budget and will report this intentional incompatibility.')
props['assets']=[];layout['instances']=[]
for aid,prefix,count,dims,front,attach,ref in groups:
    props['assets'].append(dict(id=aid,function={'desk':'Conversation surface','chair':'Seating facing conversation partner','stationery':'Exam materials and recording props','examiner-envelope':'Reserved animated examiner envelope; layout proxy only','ceiling-fixture':'Visible ceiling light support'}[aid],description='Function-stage procedural design' if aid!='examiner-envelope' else 'Fictional adult white female examiner, blonde short hair, formal charcoal suit. Current geometry is an unrigged massing proxy.',**{'class':'hero' if aid=='examiner-envelope' else 'reusable'},dimensionsMeters=dims,attachmentFace=attach,semanticFront=front,intendedInstances=count,targetPolycount=120000 if aid=='examiner-envelope' else 3000,rejectionCriteria=['Wrong physical scale or facing','Floating support or visible intersections','Not matching approved design'],image={'source':ref,'generator':'built-in image_gen' if aid=='examiner-envelope' else 'Blender actual geometry render','prompt':'Full uncropped blonde adult female examiner, charcoal formal suit; see examiner-reference-prompt.txt' if aid=='examiner-envelope' else 'Procedural geometry, not AI generation','approved':False,'approvedBy':''},attempts=[],productionMethod='character-route-pending' if aid=='examiner-envelope' else 'Blender procedural',license='source-authored' if aid!='examiner-envelope' else 'generated visual reference only; 3D model license pending'))
    prefixes=['IELTS_ExaminerChair_','IELTS_CandidateChair_'] if aid=='chair' else [prefix]
    for i,pr in enumerate(prefixes):
        parts=[o['id'] for o in geometry if o['id'].startswith(pr)]
        if aid=='stationery':parts=['IELTS_Question_Card','IELTS_Notepad','IELTS_Pen','IELTS_Recorder']
        layout['instances'].append({'id':aid+str(i+1),'assetId':aid,'objectNames':parts,'location':[0,.95 if i==0 else -1.05,0] if aid=='chair' else [0,0,0]})
save('props.json',props);save('room-layout.json',layout)
meta=load('plan-metadata.json')
meta['coverage']=['room-outline','cutouts','openings','exterior-apron','main-sign','ingress-props','floor-markers','barriers','columns','circulation','placed-features']
meta['coverageNotes']={'main-sign':'not applicable: menu arrival directly to desk','floor-markers':'not applicable: no walking gameplay','barriers':'continuous room shell','columns':'none in this small room','exterior-apron':'backed corridor outside D01','circulation':'clear side aisles; seated runtime'}
meta['sectionHeightMeters']=1.2
meta['objectGeometry']='room-layout.json:geometryObjects'
save('plan-metadata.json',meta)
openings=load('openings.json')
for o in openings['openings']:
    o['kind']=o.pop('type',o.get('kind'))
    o.update(mask=f"masks/{o['id']}.png",cutter=f"masks/{o['id']}.json",approved=False)
save('openings.json',openings)
reviews=load('milestone-reviews.json')
reviews['function'].update(status='pending',evidence=['renders/layout-production.png','renders/layout-overview.png','plans/floor-plan.png','plans/ceiling-plan.png','images/examiner-reference.png','masks/opening-contact-sheet.png'],strangestElement='Neutral articulated mannequin is visibly provisional; it promises only a scale envelope, not the final examiner.',notes='Spec and implementation plan approved in chat. Layout, visual reference and opening sheet have not yet been approved.')
save('milestone-reviews.json',reviews)
save('reference-provenance.json',{'examiner':{'generator':'built-in image_gen','source':'images/examiner-reference.png','promptFile':'images/examiner-reference-prompt.txt','role':'visual reference only, not 3D asset or rig','approved':False},'furniture':{'generator':'Blender procedural geometry','sources':['images/desk-reference.png','images/chair-reference.png']}})
(ROOT/'docs/progress.md').write_text('''# IELTS project progress

Project root: D:/雅思. Do not write project outputs to F:/Goback.

- Design specification approved by user.
- Implementation plan approved by user; execute in this chat.
- Task 1: Layout scene created; review renders, mesh-derived plans and examiner visual reference prepared. Awaiting layout/reference approval.
- Final examiner model, skinning, facial rig and speech animation have not been made.
- No paid 3D generation was submitted.
- Function validator assumes Meshy assets and a positive credit ceiling, conflicting with this project's zero-spend procedural furniture. Preserve the truthful zero budget; report incompatibility rather than fabricate pricing or approval.
- No new feature tests are needed for reversible layout-only edits; inspect geometry and saved captures.
- Original scene is preserved in backups/pre-ielts.blend and in the multi-scene layout source.
''',encoding='utf-8')
print('Review manifests saved:',PACK)
