"""Create original developer media, never a real/AI-generated IELTS paper."""
from pathlib import Path
import json, wave, math, struct, hashlib
from PIL import Image, ImageDraw
root = Path(__file__).resolve().parents[1] / 'app/test/fixtures'
media = root / 'media'
media.mkdir(exist_ok=True)
wav = media / 'three-test-tones.wav'
rate = 16000
with wave.open(str(wav), 'wb') as f:
    f.setparams((1, 2, rate, 0, 'NONE', 'not compressed'))
    frames = []
    for i in range(rate * 3):
        t = i / rate
        local = next((t - start for start in (.4, 1.2, 2.) if start <= t < start + .25), None)
        signal = 0 if local is None else .2 * math.sin(2 * math.pi * 660 * local) * min(1, local / .015, (.25-local) / .015)
        frames.append(struct.pack('<h', round(32767 * signal)))
    f.writeframes(b''.join(frames))
def asset(name, path, mime):
    return dict(id=name, path='media/'+path.name, sha256=hashlib.sha256(path.read_bytes()).hexdigest(), mediaType=mime)
base = json.loads((root/'dev-reading.json').read_text(encoding='utf-8'))
base.update(id='dev-listening', title='开发验证卷 · 听音设备测试', assets=[asset('test-tones', wav, 'audio/wav')])
base['sections'] = [dict(section='listening', title='Audio device check — test tones only', durationMs=180000, timingSource='开发设备测试 3 分钟，不是完整 IELTS 听力', paragraphs=[dict(id='l-p1', part=1, text='This developer audio contains test tones, not IELTS speech. Count the tones to check audio playback and answer saving.')], audioAssetIds=['test-tones'], questions=[dict(id='l1', number=1, kind='short_answer', part=1, points=1, prompt='How many test tones do you hear? Write ONE NUMBER.', answer=dict(accepted=['3'],caseSensitive=False,maxWords=1,maxNumbers=1), evidence=['three-test-tones.wav: 0.4s, 1.2s, 2.0s'])])]
(root/'dev-listening.json').write_text(json.dumps(base,ensure_ascii=False,indent=2),encoding='utf-8')
im=Image.new('RGB',(900,520),'#fbfcf6');d=ImageDraw.Draw(im)
d.text((45,25),'Library visitors by weekday (original developer data)',fill='#243b2c')
for i,(label,value) in enumerate(zip(['Mon','Tue','Wed','Thu','Fri'],[120,160,140,180,220])):
    x=120+i*145;top=440-value*1.5
    d.rectangle((x,top,x+85,440),fill='#638268');d.text((x+22,top-22),str(value),fill='#243b2c');d.text((x+20,460),label,fill='#243b2c')
chart=media/'library-visitors.png';im.save(chart)
writing=json.loads((root/'dev-writing.json').read_text(encoding='utf-8'));writing['assets']=[asset('visitors-chart',chart,'image/png')];writing['sections'][0]['questions'][0]['imageAssetId']='visitors-chart'
(root/'dev-writing.json').write_text(json.dumps(writing,ensure_ascii=False,indent=2),encoding='utf-8')
