"""Drive the examiner with a fixed offline speech fixture, independently of cloud APIs."""
import bpy,json,math,wave
from pathlib import Path
from collections import defaultdict
from mathutils import Quaternion
from mpfb.services.faceservice import MICROSOFT_VISEMES,ARKIT_FACEUNITS
ROOT=Path('D:/雅思');s=bpy.data.scenes['IELTS_Speaking_Form'];bpy.context.window.scene=s
body=bpy.data.objects['Examiner_Body'];rig=bpy.data.objects['Examiner_Rig']
audio=ROOT/'assets/audio/rig-test.wav'
data=json.loads((ROOT/'assets/audio/rig-test-visemes.json').read_text(encoding='utf-8-sig'))
with wave.open(str(audio)) as w:duration=w.getnframes()/w.getframerate()
assert max(e['time_ms']+e['duration_ms'] for e in data['events'])<=duration*1000+15,'Audio and viseme clocks differ'
fps=24;s.render.fps=fps;s.frame_start=1;s.frame_end=math.ceil(duration*fps)+1
meshes=[o for o in rig.children if o.type=='MESH' and o.data.shape_keys]
map_id={int(name.rsplit('_',1)[1]):name for name in MICROSOFT_VISEMES}
groups=defaultdict(list)
for e in data['events']:groups[e['time_ms']].append(e['viseme'])
for obj in meshes:
    keys=obj.data.shape_keys
    keys.animation_data_clear()
    for name in MICROSOFT_VISEMES+ARKIT_FACEUNITS:
        if name in keys.key_blocks:keys.key_blocks[name].value=0
    for ms,visemes in sorted(groups.items()):
        f=round(ms*fps/1000)+1
        for name in MICROSOFT_VISEMES:
            k=keys.key_blocks.get(name)
            if not k:continue
            active=[map_id[v] for v in visemes if v in map_id and v!=0]
            k.value=.65/len(active) if name in active else 0
            k.keyframe_insert(data_path='value',frame=f)
    for name in MICROSOFT_VISEMES:
        k=keys.key_blocks.get(name)
        if k:k.value=0;k.keyframe_insert(data_path='value',frame=s.frame_end)
    for name in ['eyeBlinkLeft','eyeBlinkRight']:
        k=keys.key_blocks.get(name)
        if not k:continue
        for frame,value in [(1,0),(43,0),(46,.95),(49,0),(107,0),(110,.95),(113,0),(s.frame_end,0)]:
            k.value=value;k.keyframe_insert(data_path='value',frame=frame)
    # Low expression weight is independent from speech and blink controls.
    for name in ['mouthSmileLeft','mouthSmileRight']:
        k=keys.key_blocks.get(name)
        if k:k.value=.08
    if keys.animation_data and keys.animation_data.action:
        keys.animation_data.action.name='Examiner_SpeechAndBlink_'+obj.name

pose=json.loads((ROOT/'assets/characters/examiner/seated-pose.json').read_text(encoding='utf-8'))
rig.animation_data_clear()
for frame in range(1,s.frame_end+1,12):
    t=(frame-1)/fps
    for name,axis,amount in [('spine02',(1,0,0),.006),('neck03',(0,0,1),.012),('head',(1,0,0),.008)]:
        p=rig.pose.bones[name];p.rotation_mode='QUATERNION'
        p.rotation_quaternion=Quaternion(pose['rotationQuaternions'][name])@Quaternion(axis,math.sin(t*1.6)*amount)
        p.keyframe_insert(data_path='rotation_quaternion',frame=frame)

ed=s.sequence_editor_create();strips=getattr(ed,'strips',None)
if strips is None:strips=ed.sequences
for strip in list(strips):
    if strip.name=='Rig_Test_Speech':strips.remove(strip)
strip=strips.new_sound('Rig_Test_Speech',str(audio),channel=1,frame_start=1)
strip.volume=.85
s.frame_set(1)
manifest={'purpose':data['purpose'],'voice':data['voice'],'sampleRate':16000,'durationSeconds':duration,'fps':fps,'frames':[s.frame_start,s.frame_end],'audio':'assets/audio/rig-test.wav','visemeEvents':len(data['events']),'bones':len(rig.data.bones),'microsoftVisemes':len(MICROSOFT_VISEMES),'arkitControls':len(ARKIT_FACEUNITS),'meshShapeKeys':{o.name:len(o.data.shape_keys.key_blocks) for o in meshes},'cloudTtsConnected':False}
(ROOT/'assets/characters/examiner/animation-validation.json').write_text(json.dumps(manifest,ensure_ascii=False,indent=2),encoding='utf-8')
for image in bpy.data.images:
    if image.source=='FILE' and image.users>0:
        try:image.pack()
        except RuntimeError:pass
bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'blender/speaking_room.blend'))
print('ANIMATION_VALIDATION_READY',json.dumps(manifest,ensure_ascii=False))
