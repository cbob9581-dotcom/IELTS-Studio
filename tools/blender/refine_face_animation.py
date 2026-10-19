"""Sample bounded coarticulation and restrained expression at the actual 24 FPS."""
import bpy,json,math,re
from pathlib import Path
ROOT=Path('D:/雅思');s=bpy.data.scenes['IELTS_Speaking_Form'];bpy.context.window.scene=s
data=json.loads((ROOT/'assets/audio/rig-test-visemes.json').read_text(encoding='utf-8-sig'))
body=bpy.data.objects['Examiner_Body'];rig=bpy.data.objects['Examiner_Rig']
speech=[k.name for k in body.data.shape_keys.key_blocks if re.search(r'_\d\d$',k.name)]
mapping={int(n.rsplit('_',1)[1]):n for n in speech}
controls=['mouthSmileLeft','mouthSmileRight','eyeBlinkLeft','eyeBlinkRight','eyeSquintLeft','eyeSquintRight','browInnerUp','eyeLookInLeft','eyeLookOutRight','eyeLookOutLeft','eyeLookInRight']
def smooth(t):t=max(0,min(1,t));return t*t*(3-2*t)
def envelope(t,start,end):
    return smooth((t-(start-.025))/.05)*(1-smooth((t-end)/.07))
rows=[]
for f in range(1,s.frame_end+1):
    t=(f-1)/s.render.fps;weights={n:0.0 for n in speech}
    for e in data['events']:
        vi=e['viseme'];name=mapping.get(vi)
        if not name or vi==0:continue
        start=e['time_ms']/1000;end=start+min(e['duration_ms']/1000,.25)
        strength=.56 if vi==21 else .30 if vi in [19,20,15] else .38
        weights[name]+=strength*envelope(t,start,end)
    total=sum(weights.values())
    if total>.62:weights={n:w*.62/total for n,w in weights.items()}
    if f==s.frame_end:weights={n:0.0 for n in weights}
    activity=min(1,sum(weights.values())/.4)
    blink=0
    for close,opened in [(1.42,1.66),(4.43,4.67)]:
        mid=close+.075
        blink=max(blink,smooth((t-close)/.075)*(1-smooth((t-mid)/(opened-mid))))
    expression={n:0.0 for n in controls}
    expression.update(mouthSmileLeft=.024*(1-.55*activity),mouthSmileRight=.022*(1-.55*activity),eyeBlinkLeft=blink,eyeBlinkRight=blink*.985,eyeSquintLeft=.035,eyeSquintRight=.035,browInnerUp=.006)
    # Small gaze shift in a pause; the main gaze stays with the candidate.
    gaze=.035*envelope(t,1.08,1.24);expression['eyeLookInLeft']=gaze;expression['eyeLookOutRight']=gaze
    rows.append((f,weights,expression))
for o in [x for x in rig.children if x.type=='MESH' and x.data.shape_keys]:
    keys=o.data.shape_keys
    if not any(n in keys.key_blocks for n in speech+controls):continue
    keys.animation_data_clear()
    for f,weights,expression in rows:
        for n,value in {**weights,**expression}.items():
            k=keys.key_blocks.get(n)
            if k:k.value=value;k.keyframe_insert(data_path='value',frame=f)
    action=keys.animation_data.action;action.name='Examiner_Coarticulation_'+o.name
    for layer in action.layers:
        for strip in layer.strips:
            for slot in action.slots:
                bag=strip.channelbag(slot)
                if bag:
                    for fc in bag.fcurves:
                        for point in fc.keyframe_points:point.interpolation='LINEAR'
s.frame_set(1)
info={'frames':len(rows),'maximumCombinedSpeechWeight':max(sum(w.values()) for _,w,_ in rows),'endSpeechWeight':sum(rows[-1][1].values()),'method':'per-frame duration-aware smooth coarticulation, restrained smile, asymmetric eyelid amplitude','audioSource':data['purpose'],'productionCloudConnected':False}
assert info['maximumCombinedSpeechWeight']<=.620001 and info['endSpeechWeight']==0
(ROOT/'assets/characters/examiner/face-refinement-inspection.json').write_text(json.dumps(info,indent=2),encoding='utf-8');print('FACE_ANIMATION_REFINED',json.dumps(info))
