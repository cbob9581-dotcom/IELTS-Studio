"""Decode the current real Blender movie and retain review evidence."""
import cv2,json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent;version=sys.argv[1] if len(sys.argv)>1 else 'v4';p=ROOT/('rooms/speaking/renders/rig-speech-preview-'+version+'.mp4')
cap=cv2.VideoCapture(str(p));assert cap.isOpened(),'Cannot open review movie'
info={'file':str(p),'frames':int(cap.get(cv2.CAP_PROP_FRAME_COUNT)),'fps':cap.get(cv2.CAP_PROP_FPS),'width':int(cap.get(cv2.CAP_PROP_FRAME_WIDTH)),'height':int(cap.get(cv2.CAP_PROP_FRAME_HEIGHT)),'bytes':p.stat().st_size,'aacTrackPresent':b'mp4a' in p.read_bytes(),'decodedFrames':[],'audioPurpose':'offline rig fixture; not Qwen production voice'}
for f in [1,15,46,110,147]:
    cap.set(cv2.CAP_PROP_POS_FRAMES,f-1);ok,frame=cap.read();assert ok,f'Frame {f} cannot be decoded'
    assert frame.std()>5,'Unexpected blank image';info['decodedFrames'].append(f)
    if f==15:
        ok,b=cv2.imencode('.png',frame);assert ok;(p.parent/('rig-video-'+version+'-frame15.png')).write_bytes(b.tobytes())
cap.release();assert info['frames']==147 and info['fps']==24 and info['aacTrackPresent']
(ROOT/'assets/characters/examiner/hair-movie-inspection.json').write_text(json.dumps(info,ensure_ascii=False,indent=2),encoding='utf-8')
reportPath=ROOT/'rooms/speaking/final-report.json';report=json.loads(reportPath.read_text(encoding='utf-8'));report['reviewRenders']['movie']='renders/rig-speech-preview-'+version+'.mp4';reportPath.write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(info,ensure_ascii=True))
