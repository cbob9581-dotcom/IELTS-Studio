"""Fetch public MPFB source and CC0 character packs into the IELTS project."""
from pathlib import Path
import concurrent.futures
import hashlib
import json
import sys
import urllib.request
import zipfile
sys.stdout.reconfigure(encoding='utf-8')
ROOT=Path(__file__).resolve().parents[1]
OUT=ROOT/'.tools/character-sources'
OUT.mkdir(parents=True,exist_ok=True)
SOURCES={
 'mpfb2-source.zip':'https://codeload.github.com/makehumancommunity/mpfb2/zip/refs/heads/master',
 'makehuman_system_assets_cc0.zip':'https://files.makehumancommunity.org/asset_packs/makehuman_system_assets/makehuman_system_assets_cc0.zip',
 'suits01_cc0.zip':'https://files.makehumancommunity.org/asset_packs/suits01/suits01_cc0.zip',
 'visemes01.zip':'https://files.makehumancommunity.org/functional/visemes01.zip',
 'faceunits01.zip':'https://files.makehumancommunity.org/functional/faceunits01.zip',
}
def fetch(item):
 name,url=item;p=OUT/name
 if not p.exists():
  req=urllib.request.Request(url,headers={'User-Agent':'IELTS-Blender-Asset-Preparation'})
  with urllib.request.urlopen(req,timeout=90) as r,p.with_suffix('.partial').open('wb') as f:
   while chunk:=r.read(1024*1024):f.write(chunk)
  p.with_suffix('.partial').replace(p)
 with zipfile.ZipFile(p) as z:
  bad=z.testzip()
  if bad:raise RuntimeError('Broken archive '+bad)
 result={'file':str(p.relative_to(ROOT)),'url':url,'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest(),'assetLicense':'GPL source, CC0 bundled core assets' if name.startswith('mpfb2') else 'CC0'}
 print('READY',name,result['bytes'],flush=True)
 return result
with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
 records=list(pool.map(fetch,SOURCES.items()))
(OUT/'sources.json').write_text(json.dumps(records,ensure_ascii=False,indent=2),encoding='utf-8')
source=OUT/'mpfb2-source.zip'
dest=ROOT/'.tools/mpfb-source'
with zipfile.ZipFile(source) as z:
 for member in z.infolist():
  if member.is_dir():continue
  # Only official source/data/docs; never execute downloaded files during extraction.
  relative=Path(*Path(member.filename).parts[1:])
  target=(dest/relative).resolve()
  if not target.is_relative_to(dest.resolve()):raise RuntimeError('Unsafe archive path')
  target.parent.mkdir(parents=True,exist_ok=True);target.write_bytes(z.read(member))
print('SOURCES_READY',dest)
