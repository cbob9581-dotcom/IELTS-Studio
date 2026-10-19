"""Independent Part 2 handoff scene, rigged reach, grasp and prop release."""
import bpy,math,json
from pathlib import Path
from mathutils import Vector,Matrix,Quaternion
ROOT=Path('D:/雅思');base=bpy.data.scenes['IELTS_Speaking_Form'];base.frame_set(1)
old=bpy.data.scenes.get('IELTS_Part2_Handoff')
if old:
    for o in list(old.objects):
        if o.name.startswith('P2_'):bpy.data.objects.remove(o,do_unlink=True)
    bpy.data.scenes.remove(old)
for action in list(bpy.data.actions):
    if action.name.startswith('Part2_Card_Paper_Pen_Handoff') and action.users==0:bpy.data.actions.remove(action)
s=bpy.data.scenes.new('IELTS_Part2_Handoff');s.world=base.world;s.render.engine=base.render.engine;s.render.fps=24;s.frame_start=1;s.frame_end=336
s.render.resolution_x=1280;s.render.resolution_y=720;s.render.resolution_percentage=100
exclude={'IELTS_Question_Card','IELTS_Notepad','IELTS_Pen'}
for o in base.objects:
    if not o.name.startswith('Examiner_') and o.get('logical_id') not in exclude and o.type!='CAMERA':s.collection.objects.link(o)
source_rig=bpy.data.objects['Examiner_Rig'];rig=source_rig.copy();rig.data=source_rig.data.copy();rig.name='P2_Examiner_Rig';rig.animation_data_clear();s.collection.objects.link(rig)
character=[]
for o in base.objects:
    if o.name.startswith('Examiner_') and o.type=='MESH':
        c=o.copy();c.data=o.data.copy();c.name='P2_'+o.name;c.animation_data_clear();c.parent=rig
        for m in c.modifiers:
            if m.type=='ARMATURE':m.object=rig
        if c.data.shape_keys:
            c.data.shape_keys.animation_data_clear()
            for k in c.data.shape_keys.key_blocks:
                if k.name in ['mouthSmileLeft','mouthSmileRight']:k.value=.025
                elif k.name.startswith(('eyeBlink','eyeLook')) or k.name.rsplit('_',1)[-1].isdigit():k.value=0
        s.collection.objects.link(c);character.append(c)
cam=base.camera.copy();cam.data=base.camera.data.copy();cam.name='P2_CandidateCamera';s.collection.objects.link(cam);cam.location=(0,-1.20,1.18);cam.rotation_euler=(Vector((0,.28,.96))-cam.location).to_track_quat('-Z','Y').to_euler();cam.data.lens=38;s.camera=cam
bpy.context.window.scene=s;s.frame_set(1)
rest={p.name:p.rotation_quaternion.copy() for p in rig.pose.bones};arm_names={side:[n+'.'+side for n in ['upperarm01','upperarm02','lowerarm01','lowerarm02','wrist']]+[p.name for p in rig.pose.bones if p.name.startswith('finger') and p.name.endswith('.'+side)] for side in ['R','L']}
rest_wrist={side:(rig.matrix_world@rig.pose.bones['wrist.'+side].head).copy() for side in ['R','L']}
rest_wrist_rotation={side:rig.pose.bones['wrist.'+side].matrix.to_quaternion().copy() for side in ['R','L']}
def material(name,color,rough=.6):
    m=bpy.data.materials.get(name) or bpy.data.materials.new(name);m.use_nodes=True;p=next(n for n in m.node_tree.nodes if n.type=='BSDF_PRINCIPLED');p.inputs['Base Color'].default_value=(*color,1);p.inputs['Roughness'].default_value=rough;return m
ivory=material('P2_CardStock',(.83,.82,.74));paper=material('P2_WhitePaper',(.92,.93,.94));ink=material('P2_PrintInk',(.018,.025,.035));black=material('P2_PenBarrel',(.018,.025,.035),.25);silver=material('P2_PenMetal',(.42,.47,.52),.20)
def box(name,size,location,mat):
    x,y,z=[a/2 for a in size];vs=[(-x,-y,-z),(-x,-y,z),(-x,y,-z),(-x,y,z),(x,-y,-z),(x,-y,z),(x,y,-z),(x,y,z)];fs=[(0,4,6,2),(1,3,7,5),(0,1,5,4),(2,6,7,3),(0,2,3,1),(4,5,7,6)]
    me=bpy.data.meshes.new(name);me.from_pydata(vs,[],fs);me.materials.append(mat);o=bpy.data.objects.new(name,me);s.collection.objects.link(o);o.location=location;return o
card=box('P2_TopicCard',(.150,.210,.0012),(-.22,.22,.760),ivory)
sheet=box('P2_BlankPaper',(.148,.210,.0005),(.23,.22,.760),paper)
for x in [-.22,.23]:box('P2_SourcePaperStack_'+str(x),(.14,.12,.017),(x,.205,.749),paper)
def text(parent,body,x,y,size):
    d=bpy.data.curves.new('P2_CardText','FONT');d.body=body;d.size=size;d.extrude=0;d.space_line=1.15;o=bpy.data.objects.new('P2_CardText',d);s.collection.objects.link(o);o.parent=parent;o.location=(x,y,.00075);d.materials.append(ink)
text(card,'SPEAKING  /  PART 2',-.064,.083,.007)
text(card,'Describe a place you\nenjoyed visiting.',-.064,.061,.006)
text(card,'You should say:\n\n- where it is\n- who you went with\n- what you did there\n\nand explain why you\nenjoyed visiting it.',-.064,.031,.0048)
text(card,'Original practice topic',-.064,-.091,.0038)
pen=box('P2_Pen',(.140,.006,.006),(.35,.22,.758),black)
for x in [.30,.40]:box('P2_PenRest_'+str(x),(.012,.028,.014),(x,.22,.747),black)
def cylinder_mesh(name,length,r0,r1):
    vs=[];fs=[];n=24
    for x,r in [(-length/2,r0),(length/2,r1)]:
        for i in range(n):a=i/n*2*math.pi;vs.append((x,r*math.cos(a),r*math.sin(a)))
    for i in range(n):j=(i+1)%n;fs.append((i,j,j+n,i+n))
    fs.extend([tuple(reversed(range(n))),tuple(range(n,n*2))]);me=bpy.data.meshes.new(name);me.from_pydata(vs,[],fs);me.update();return me
pen.data=cylinder_mesh('P2_RoundPenBarrel',.14,.003,.003);pen.data.materials.append(black)
tip=box('P2_PenTip',(.012,.003,.003),(0,0,0),silver);tip.parent=pen;tip.location=(-.076,0,0)
tip.data=cylinder_mesh('P2_TaperedPenTip',.012,.0003,.003);tip.data.materials.append(silver)
clip=box('P2_PenClip',(.040,.0015,.0015),(0,0,0),silver);clip.parent=pen;clip.location=(.042,0,.004)
props={'card':card,'paper':sheet,'pen':pen};bpy.context.view_layer.update();starts={k:o.matrix_world.copy() for k,o in props.items()}
def smooth(t):t=max(0,min(1,t));return t*t*(3-2*t)
def track(points,f):
    if f<=points[0][0]:return Vector(points[0][1])
    for (a,p),(b,q) in zip(points,points[1:]):
        if f<=b:return Vector(p).lerp(Vector(q),smooth((f-a)/(b-a)))
    return Vector(points[-1][1])
def aim(name,direction):
    p=rig.pose.bones[name];delta=(p.tail-p.head).normalized().rotation_difference(Vector(direction).normalized());p.matrix=Matrix.LocRotScale(p.head.copy(),delta@p.matrix.to_quaternion(),Vector((1,1,1)));bpy.context.view_layer.update()
def solve_arm(side,wrist,lean,blend=1):
    # Analytic shoulder/elbow solve; MPFB divides each limb into two deform bones.
    names=[n+'.'+side for n in ['upperarm01','upperarm02','lowerarm01','lowerarm02']];p=rig.pose.bones[names[0]];shoulder=(rig.matrix_world@p.head).copy();a=sum(rig.pose.bones[n].length*rig.scale.x for n in names[:2]);b=sum(rig.pose.bones[n].length*rig.scale.x for n in names[2:]);d=wrist-shoulder;distance=d.length;direction=d.normalized();reach=min(distance,a+b-.003)
    pole=Vector((-.35 if side=='R' else .35,.02,-1));desired=(pole-direction*pole.dot(direction)).normalized();rest_elbow=rig.matrix_world@rig.pose.bones[names[2]].head;rp=rest_elbow-shoulder;start=(rp-direction*rp.dot(direction)).normalized();perp=start.lerp(desired,blend).normalized();along=(a*a-b*b+reach*reach)/(2*reach);height=math.sqrt(max(0,a*a-along*along));elbow=shoulder+direction*along+perp*height
    inv=rig.matrix_world.inverted().to_3x3()
    for name in names[:2]:aim(name,inv@(elbow-shoulder))
    for name in names[2:]:aim(name,inv@(wrist-elbow))
    # Keep palm down and fingers aimed toward the candidate.
    pb=rig.pose.bones['wrist.'+side];sign=1 if side=='L' else -1;forward=Vector((0,-1,-.03)).normalized();up=Vector((0,0,1));x=(-sign*(up-forward*up.dot(forward))).normalized();rot=rest_wrist_rotation[side].slerp(Matrix((x,forward,x.cross(forward))).transposed().to_quaternion(),blend);pb.matrix=Matrix.LocRotScale(pb.head.copy(),rot,Vector((1,1,1)));bpy.context.view_layer.update();return max(0,distance-a-b)
def finger_ccd(side,digit,target):
    inv=rig.matrix_world.inverted();target=inv@target;chain=[f'finger{digit}-{j}.{side}' for j in [1,2,3]]
    for iteration in range(7):
        for name in reversed(chain):
            p=rig.pose.bones[name];end=rig.pose.bones[chain[-1]].tail;d1=end-p.head;d2=target-p.head
            if d1.length<1e-6 or d2.length<1e-6:continue
            delta=d1.normalized().rotation_difference(d2.normalized());p.matrix=Matrix.LocRotScale(p.head.copy(),delta@p.matrix.to_quaternion(),Vector((1,1,1)));bpy.context.view_layer.update()
grips={}
for side in ['R','L']:
    basegrip=bpy.data.objects.new('P2_WristFrame_'+side,None);s.collection.objects.link(basegrip);con=basegrip.constraints.new('COPY_TRANSFORMS');con.target=rig;con.subtarget='wrist.'+side;grips[side]=basegrip
tasks=[{'name':'card','side':'R','reach':17,'pick':45,'lift':63,'place':89,'release':91,'end':112,'drop':(-.13,.040,.7410)}, {'name':'paper','side':'L','reach':113,'pick':137,'lift':153,'place':177,'release':179,'end':196,'drop':(.14,.055,.7407)}, {'name':'pen','side':'L','reach':197,'pick':219,'lift':235,'place':257,'release':259,'end':288,'drop':(.18,.150,.7440)}]
for task in tasks:
    name=task['name'];o=props[name];a=bpy.data.objects.new('P2_Grip_'+name,None);s.collection.objects.link(a);a.parent=grips[task['side']];con=o.constraints.new('COPY_TRANSFORMS');con.name='P2_HeldByHand';con.target=a;con.influence=0;task['anchor']=a;task['constraint']=con;task['start']=Vector(o.location);task['offset']=Vector((0,.215 if name!='pen' else .110,.037 if name!='pen' else .034))
events=[];maxreach=0
# Defer expensive character skinning until the baked control transforms are ready.
visibility=[(o,o.hide_viewport) for o in character]
for o,_ in visibility:o.hide_viewport=True
for f in range(1,337):
    s.frame_set(f)
    for name,q in rest.items():rig.pose.bones[name].rotation_quaternion=q
    active=next((t for t in tasks if t['reach']<=f<=t['end']),None)
    lean=0
    if active:
        t=active;lean=.32*smooth((f-t['reach'])/22)*(1-smooth((f-t['release']-6)/(t['end']-t['release']-6)))
    arm_blend=smooth((f-active['reach'])/8)*(1-smooth((f-active['end']+8)/8)) if active else 0
    for name,amount in [('spine02',lean*.65),('spine01',lean*.35)]:rig.pose.bones[name].rotation_quaternion=rest[name]@Quaternion((1,0,0),amount)
    bpy.context.view_layer.update()
    for side in ['R','L']:
        wrist=rest_wrist[side].copy();grasp=0;task=active if active and active['side']==side else None
        if task:
            src=task['start'];dst=Vector(task['drop']);off=task['offset'];lift=src+Vector((0,-.045,.105));near=dst+Vector((0,0,.080))
            points=[(task['reach'],rest_wrist[side]),(task['pick']-4,src+off),(task['pick'],src+off),(task['lift'],lift+off),(task['place']-10,near+off),(task['place'],dst+off),(task['release'],dst+off),(task['release']+8,dst+off+Vector((0,.025,.08))),(task['end'],rest_wrist[side])]
            wrist=track(points,f);grasp=smooth((f-task['pick']+7)/7)*(1-smooth((f-task['release']+1)/6))
        if task or lean>1e-5:maxreach=max(maxreach,solve_arm(side,wrist,lean,arm_blend))
        if task and grasp>.001:
            # Tip bones straddle the thin prop; the thumb stays above the tabletop.
            center=wrist-task['offset'];back=.104 if task['name']!='pen' else 0
            grasp*=smooth((center.z-.746)/.012)
            index_target=center+Vector((-.012 if side=='R' else .012,back,.009 if task['name']=='pen' else .007));thumb_target=center+Vector((-.010 if side=='R' else .010,back+.006,-.007 if task['name']=='pen' else -.006))
            before={n:rig.pose.bones[n].rotation_quaternion.copy() for n in arm_names[side] if n.startswith('finger')}
            finger_ccd(side,2,index_target);finger_ccd(side,1,thumb_target)
            for n,q in before.items():rig.pose.bones[n].rotation_quaternion=q.slerp(rig.pose.bones[n].rotation_quaternion,grasp)
            bpy.context.view_layer.update()
    # Gaze follows the task briefly; face remains quiet until a future cloud voice cue.
    attention=smooth((f-active['reach'])/10)*(1-smooth((f-active['end']+10)/10)) if active else 0
    look=.07*attention;rig.pose.bones['head'].rotation_quaternion=rest['head']@Quaternion((1,0,0),look)
    blink=max([math.sin(math.pi*(f-start)/6)**2 if start<f<start+6 else 0 for start in [8,99,190,283]])
    for obj in character:
        if not obj.data.shape_keys:continue
        keys=obj.data.shape_keys.key_blocks
        for name,value in [('eyeBlinkLeft',blink),('eyeBlinkRight',blink*.99),('eyeLookDownLeft',.10*attention),('eyeLookDownRight',.10*attention)]:
            key=keys.get(name)
            if key:key.value=value;key.keyframe_insert(data_path='value',frame=f)
    for name in ['spine02','spine01','head']+arm_names['R']+arm_names['L']:rig.pose.bones[name].keyframe_insert(data_path='rotation_quaternion',frame=f)
    bpy.context.view_layer.update()
    for task in tasks:
        o=props[task['name']];con=task['constraint'];a=task['anchor']
        if f==task['pick']:
            bpy.context.view_layer.update();a.matrix_parent_inverse=Matrix.Identity(4);a.matrix_basis=grips[task['side']].matrix_world.inverted()@starts[task['name']];events.append({'frame':f,'event':'attach','prop':task['name'],'hand':task['side']})
        held=task['pick']<=f<task['release'];con.influence=1 if held else 0;con.keyframe_insert(data_path='influence',frame=f)
        if f<task['pick']:o.matrix_world=starts[task['name']]
        elif f==task['release']:
            bpy.context.view_layer.update();o.matrix_world=a.matrix_world.copy();task['releasedMatrix']=o.matrix_world.copy();events.append({'frame':f,'event':'release','prop':task['name'],'hand':task['side']})
        elif f>task['release']:
            o.matrix_world=task['releasedMatrix'].copy();o.location.z+=(task['drop'][2]-o.location.z)*smooth((f-task['release'])/6)
            if f==task['release']+6:events.append({'frame':f,'event':'settled','prop':task['name']})
        if f<=task['pick'] or f>=task['release']:
            o.keyframe_insert(data_path='location',frame=f);o.keyframe_insert(data_path='rotation_euler',frame=f)
    if f==289:events.append({'frame':f,'event':'preparation_ready'})
def linear(action):
    if not action:return
    for layer in action.layers:
        for strip in layer.strips:
            for slot in action.slots:
                bag=strip.channelbag(slot)
                if bag:
                    for fc in bag.fcurves:
                        for p in fc.keyframe_points:p.interpolation='LINEAR'
linear(rig.animation_data.action)
for o in props.values():linear(o.animation_data.action)
for frame,label in [(1,'Introduce task'),(45,'Pick up topic card'),(91,'Release topic card'),(137,'Pick up blank paper'),(179,'Release paper'),(219,'Pick up pen'),(259,'Release pen'),(289,'Preparation ready')]:s.timeline_markers.new(label,frame=frame)
rig.animation_data.action.name='Part2_Card_Paper_Pen_Handoff';s.frame_set(1)
for o,hidden in visibility:o.hide_viewport=hidden
bpy.context.view_layer.update()
info={'scene':s.name,'fps':24,'frames':336,'durationSeconds':14,'events':events,'maxRequestedReachBeyondArmMeters':maxreach,'audio':'silent choreography preview; production cloud speech not connected','topic':'original practice prompt, not an official test item','runtimeExported':False}
(ROOT/'assets/characters/examiner/part2-handoff.json').write_text(json.dumps(info,indent=2),encoding='utf-8');bpy.ops.wm.save_as_mainfile(filepath=str(ROOT/'blender/speaking_room.blend'));print('PART2_HANDOFF_CREATED',json.dumps(info))
