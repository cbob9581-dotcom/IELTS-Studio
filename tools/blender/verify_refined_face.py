"""Inspect evaluated face weights and eyelash synchronization for every frame."""
import bpy,re,json
from pathlib import Path
ROOT=Path('D:/雅思');s=bpy.data.scenes['IELTS_Speaking_Form'];body=bpy.data.objects['Examiner_Body'];before=s.frame_current
meshes=[o for o in bpy.data.objects['Examiner_Rig'].children if o.type=='MESH' and o.data.shape_keys]
report={'frames':0,'maximumSpeechSum':0,'minimumAnimatedWeight':1,'maximumAnimatedWeight':0,'maximumSharedBlinkDifference':0,'endSpeechSum':None}
for f in range(s.frame_start,s.frame_end+1):
    s.frame_set(f);keys=body.data.shape_keys.key_blocks;speech=[k.value for k in keys if re.search(r'_\d\d$',k.name)];total=sum(speech)
    report['frames']+=1;report['maximumSpeechSum']=max(report['maximumSpeechSum'],total)
    values=speech+[keys[n].value for n in ['eyeBlinkLeft','eyeBlinkRight','mouthSmileLeft','mouthSmileRight']]
    report['minimumAnimatedWeight']=min(report['minimumAnimatedWeight'],min(values));report['maximumAnimatedWeight']=max(report['maximumAnimatedWeight'],max(values))
    for o in meshes:
        for name in ['eyeBlinkLeft','eyeBlinkRight']:
            k=o.data.shape_keys.key_blocks.get(name)
            if k:report['maximumSharedBlinkDifference']=max(report['maximumSharedBlinkDifference'],abs(k.value-keys[name].value))
    if f==s.frame_end:report['endSpeechSum']=total
s.frame_set(before);report['passed']=report['maximumSpeechSum']<=.62001 and report['minimumAnimatedWeight']>=0 and report['maximumAnimatedWeight']<=1 and report['maximumSharedBlinkDifference']<1e-6 and report['endSpeechSum']==0
(ROOT/'assets/characters/examiner/face-evaluated-inspection.json').write_text(json.dumps(report,indent=2),encoding='utf-8');print('EVALUATED_FACE',json.dumps(report))
if not report['passed']:raise RuntimeError('Face-weight or eyelash synchronization check failed')
