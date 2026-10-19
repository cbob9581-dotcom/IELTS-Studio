"""Record current review artifacts without inferring appearance approval."""
from pathlib import Path
import json,shutil,hashlib
ROOT=Path(__file__).resolve().parent.parent;pack=ROOT/'rooms/speaking';renders=pack/'renders'
for stem in ['face-neutral','face-vowel','face-blink','hands-table','hands-table-side','form-production']:
    shutil.copy2(renders/(stem+'-v2.png'),renders/(stem+'-v4.png'))
p=pack/'final-report.json';report=json.loads(p.read_text(encoding='utf-8'))
report['reviewRenders']['hair']='renders/hair-eight-views.png'
report['reviewRenders']['production']='renders/form-production-v4.png'
report['reviewRenders']['face']=['renders/face-'+label+'-v4.png' for label in ['neutral','vowel','blink']]
report['reviewRenders']['hands']=['renders/hands-table-v4.png','renders/hands-table-side-v4.png']
report['appearanceRevision']={'id':'hair-card-rebuild-v4','date':'2026-10-16','status':'awaiting user appearance review','hairSource':'Original procedural geometry','bindingCheck':'assets/characters/examiner/hair-binding-inspection.json','handCheck':'assets/characters/examiner/hand-table-inspection.json','runtimeExported':False}
report['notes']=['Current review uses original project-authored hair geometry; old Daisy hair is archived.','Cloud ASR, LLM and Qwen TTS are not connected. Fixed local speech is only a rig-validation fixture.','Hair and character Form approval remain pending; runtime is not exported.']
p.write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
info=json.loads((ROOT/'assets/characters/examiner/hair-rebuild.json').read_text())
info['generatorSha256']=hashlib.sha256((ROOT/'tools/blender/rebuild_hair_cards.py').read_bytes()).hexdigest()
(ROOT/'assets/characters/examiner/hair-rebuild.json').write_text(json.dumps(info,indent=2),encoding='utf-8')
print('Current stills and provenance recorded; appearance approval remains pending.')
