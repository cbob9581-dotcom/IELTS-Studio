"""Taper the lower bob toward the nape without shrinking scalp-fitting roots."""
import bpy,json
from pathlib import Path
ROOT=Path('D:/雅思')
def smooth(x):x=max(0,min(1,x));return x*x*(3-2*x)
def target(p):
    p=p.copy();lower=smooth((1.535-p.z)/.145)
    rear=smooth((p.y+.008)/.073);front=smooth((-.065-p.y)/.055)
    p.y+=lower*(.010*front-.027*rear)
    return p
changed={}
for name in ['Examiner_Hair','Examiner_HairStrands']:
    o=bpy.data.objects[name]
    if o.get('profile_depth_refined_v7'):continue
    for v in o.data.vertices:v.co=target(v.co)
    o.data.update();o['profile_depth_refined_v7']=True;changed[name]=len(o.data.vertices)
info={'method':'smooth lower-profile taper preserving scalp roots','maximumRearReductionMeters':.027,'maximumFrontRetractionMeters':.010,'blendStartRestZ':1.535,'blendEndRestZ':1.390,'modifiedObjects':changed,'rootCapUnchanged':True}
(ROOT/'assets/characters/examiner/hair-profile-depth.json').write_text(json.dumps(info,indent=2),encoding='utf-8');bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'blender/speaking_room.blend'));print('HAIR_PROFILE_TAPER',json.dumps(info))
