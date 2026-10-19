"""Load MPFB for this Blender session with all writable state in the IELTS project."""
import sys,os,json,contextlib
from pathlib import Path
ROOT=Path('D:/雅思')
HOME=ROOT/'.tools/mpfb-user'
HOME.mkdir(parents=True,exist_ok=True)
os.environ['IELTS_MPFB_HOME']=str(HOME)
source=ROOT/'.tools/mpfb-source/src'
location=source/'mpfb/services/locationservice.py'
text=location.read_text(encoding='utf-8')
old='bpy.utils.extension_path_user(MPFB_CONTEXTUAL_INFORMATION["__package__"])'
replacement='(os.environ["IELTS_MPFB_HOME"] if "IELTS_MPFB_HOME" in os.environ else '+old+')'
if 'os.environ["IELTS_MPFB_HOME"]' not in text:
    location.write_text(text.replace(old,replacement),encoding='utf-8')
if str(source) not in sys.path:sys.path.insert(0,str(source))
import mpfb
if mpfb.ClassManager is None:
    # Source distribution loaded for this session; no global addon or user-preference installation.
    prefs={'mpfb_user_data':str(HOME),'mpfb_second_root':'','mh_user_data':'','mh_auto_user_data':False,'mpfb_shelf_label':'MPFB','mpfb_codechecks':False,'mpfb_show_start_here':False}
    mpfb.get_preference=lambda key:prefs.get(key)
    with (HOME/'bootstrap.log').open('w',encoding='utf-8') as log,contextlib.redirect_stdout(log):
        mpfb.register()
from mpfb.services import LogService,LocationService,AssetService
LogService.set_default_log_level(LogService.WARN)
print(json.dumps({'mpfb_version':mpfb.VERSION,'data':LocationService.get_user_data(),'root':LocationService.get_mpfb_root()},ensure_ascii=False))
