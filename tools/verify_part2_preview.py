"""Decode every rendered frame, assemble review stages, and register the artifact."""
import cv2,numpy as np,json
from pathlib import Path
ROOT=Path(__file__).resolve().parent.parent;r=ROOT/'rooms/speaking/renders';path=r/'part2-handoff-preview.mp4';cap=cv2.VideoCapture(str(path));assert cap.isOpened()
fps=cap.get(cv2.CAP_PROP_FPS);width=int(cap.get(cv2.CAP_PROP_FRAME_WIDTH));height=int(cap.get(cv2.CAP_PROP_FRAME_HEIGHT));decoded=0;previous=None;maximumDelta=0;deltaFrame=None;tiles=[]
samples={1:'Start',63:'Topic card',89:'Place card',153:'Blank paper',235:'Pen',336:'Ready'}
while True:
    ok,frame=cap.read()
    if not ok:break
    decoded+=1;assert frame.std()>5,'Unexpected blank image'
    if previous is not None:
        delta=float(cv2.absdiff(frame,previous).mean())
        if delta>maximumDelta:maximumDelta=delta;deltaFrame=decoded
    previous=frame
    if decoded in samples:
        tile=np.full((306,480,3),245,dtype=np.uint8);tile[:270]=cv2.resize(frame,(480,270));cv2.putText(tile,samples[decoded],(12,294),cv2.FONT_HERSHEY_SIMPLEX,.6,(30,30,30),1,cv2.LINE_AA);tiles.append(tile)
cap.release();assert decoded==336 and fps==24 and width==768 and height==432
sheet=np.vstack([np.hstack(tiles[:3]),np.hstack(tiles[3:])]);ok,b=cv2.imencode('.png',sheet);assert ok;(r/'part2-handoff-storyboard.png').write_bytes(b.tobytes())
info={'framesDecoded':decoded,'fps':fps,'durationSeconds':decoded/fps,'width':width,'height':height,'bytes':path.stat().st_size,'maxMeanPixelChange':maximumDelta,'maxChangeFrame':deltaFrame,'audio':'silent choreography preview','passed':True}
(ROOT/'assets/characters/examiner/part2-movie-inspection.json').write_text(json.dumps(info,indent=2),encoding='utf-8')
reportPath=ROOT/'rooms/speaking/final-report.json';report=json.loads(reportPath.read_text(encoding='utf-8'));report['part2Handoff']={'scene':'IELTS_Part2_Handoff','action':'Part2_Card_Paper_Pen_Handoff','movie':'renders/part2-handoff-preview.mp4','storyboard':'renders/part2-handoff-storyboard.png','geometryCheck':'assets/characters/examiner/part2-handoff-inspection.json','status':'Blender choreography preview; visual approval pending','cloudSpeechConnected':False,'runtimeExported':False};reportPath.write_text(json.dumps(report,ensure_ascii=False,indent=2),encoding='utf-8')
print(json.dumps(info))
