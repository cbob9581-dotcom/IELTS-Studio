"""Download the official CC-BY hair pack, retaining author attribution and source hash."""
from pathlib import Path
import urllib.request,zipfile,hashlib,json
ROOT=Path(__file__).resolve().parents[1]
out=ROOT/'.tools/character-sources/hair02_ccby.zip'
url='https://files.makehumancommunity.org/asset_packs/hair02/hair02_ccby.zip'
if not out.exists():
    with urllib.request.urlopen(url,timeout=90) as r,out.with_suffix('.partial').open('wb') as f:
        while chunk:=r.read(1024*1024):f.write(chunk)
    out.with_suffix('.partial').replace(out)
dest=ROOT/'.tools/mpfb-user/data'
with zipfile.ZipFile(out) as z:
    for entry in z.infolist():
        if entry.is_dir():continue
        p=(dest/entry.filename).resolve()
        if not p.is_relative_to(dest.resolve()):raise RuntimeError('Unsafe archive path')
        p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(z.read(entry))
record={'author':'Elvaerwyn','pack':'hair02','url':url,'sourcePage':'https://static.makehumancommunity.org/assets/assetpacks/hair02.html','license':'CC-BY; retain asset-specific included license','sha256':hashlib.sha256(out.read_bytes()).hexdigest(),'bytes':out.stat().st_size}
(ROOT/'assets/characters/examiner/hair-pack-source.json').write_text(json.dumps(record,indent=2),encoding='utf-8')
print('HAIR_PACK_READY',out.stat().st_size)
