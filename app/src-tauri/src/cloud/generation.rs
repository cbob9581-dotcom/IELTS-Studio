use super::error;
use crate::contracts::Result;
use serde_json::{json, Value};
pub fn count(section: &str, part: u64) -> usize {
    match section {
        "reading" => {
            if part == 3 {
                14
            } else {
                13
            }
        }
        "listening" => 10,
        "writing" => 1,
        "speaking" => {
            if part == 2 {
                1
            } else {
                4
            }
        }
        _ => 0,
    }
}
pub fn question_plan(section:&str,part:u64,kind:Option<&str>)->Vec<Value> { question_plan_variant(section,part,kind,0) }
pub fn question_plan_variant(section:&str,part:u64,kind:Option<&str>,seed:u64)->Vec<Value> {
    let total=count(section,part); let mut out=Vec::new(); let mut number=1;
    while number<=total {
        let variant=if section=="reading"&&seed%3==1 {Some(if number<=6 {"heading_match"}else{"gap_fill"})}else if section=="reading"&&seed%3==2 {Some(if number<=5 {"matching"}else if number<=10 {"true_false_not_given"}else{"short_answer"})}else{None};
        let k=kind.or(variant).unwrap_or(match section {
            "listening" if part==1||part==4=>"gap_fill",
            "listening" if part==2&&number>5=>"matching",
            "listening" if part==3&&number>=7=>"gap_fill",
            "listening" if part==3&&number==5=>"multiple_choice",
            "listening"=>"single_choice",
            "reading" if part==1&&number<=5=>"true_false_not_given",
            "reading" if part==2&&number<=6=>"heading_match",
            "reading" if part==3&&number<=5=>"yes_no_not_given",
            "reading" if part==1&&number<=8||part==3&&number<=10=>"single_choice",
            "reading"=>"gap_fill", "writing"=>"writing_task", _=>"speaking_prompt"
        });
        let points=if ["writing","speaking"].contains(&section){0}else if k=="multiple_choice"{if total-number+1==3{3}else{2.min(total-number+1)}}else{1};
        let width=points.max(1);out.push(json!({"number":number,"part":part,"kind":k,"points":points,"coveredNumbers":(number..number+width).collect::<Vec<_>>()})); number+=width;
    } out
}
pub fn prompt(section: &str, part: u64, kind: Option<&str>, topic: &str) -> Result<String> { prompt_for_request(section,part,kind,topic,&json!({})) }
pub fn prompt_for_request(section: &str, part: u64, kind: Option<&str>, topic: &str,request:&Value) -> Result<String> {
    let max=match section{"listening"=>4,"reading"|"speaking"=>3,"writing"=>2,_=>0};
    if part<1||part>max||topic.len()>1000{return Err(error("科目、部分或主题无效"))}
    let allowed=match section{
        "reading"=>vec!["single_choice","multiple_choice","matching","gap_fill","short_answer","diagram_label","true_false_not_given","yes_no_not_given","heading_match"],
        "listening"=>vec!["single_choice","multiple_choice","matching","gap_fill","short_answer","diagram_label"],
        "writing"=>vec!["writing_task"],_=>vec!["speaking_prompt"]};
    if kind.is_some_and(|k|!allowed.contains(&k)){return Err(error("此科目不支持所选题型"))}
    let plan=question_plan_variant(section,part,kind,request["blueprintSeed"].as_u64().unwrap_or(0));
    let mut contracts=Vec::new();
    for k in &allowed { if !plan.iter().any(|q|q["kind"]==*k){continue;}
      contracts.push(match *k {
       "multiple_choice"=>"multiple_choice: options=[full option strings], answer={accepted:[[full option A,full option C]],caseSensitive:false}. One question object per group, points is group size.",
       "single_choice"|"heading_match"=>"single_choice/heading_match: options=[full option strings], answer={accepted:[one full correct option],caseSensitive:false}.",
       "matching"=>"matching: ONE slot per numbered item. slots=[slot text], options=[full option strings], answer={accepted:[{slot text:full correct option}],caseSensitive:false}.",
       "gap_fill"|"short_answer"=>"gap_fill/short_answer: prompt includes explicit word/number limit and complete context with ____ blank; answer={accepted:[correct response],caseSensitive:false,maxWords:3,maxNumbers:1}. Choose exact source answers of at most three words and state matching instructions.",
       "true_false_not_given"=>"true_false_not_given: answer={accepted:[TRUE or FALSE or NOT GIVEN],caseSensitive:false}. No options or slots.",
       "yes_no_not_given"=>"yes_no_not_given: answer={accepted:[YES or NO or NOT GIVEN],caseSensitive:false}. No options or slots.",
       "diagram_label"=>"diagram_label: visual={title,items:[{id,text,x,y}],connections:[{from,to}]}, all x/y normalized 0.15..0.85. Answer nodes show question number/blank, never answers. Each question has labels=[{id,x,y}], answer={accepted:[{labelId:answer}],caseSensitive:false,maxWords:2}. Application assigns imageAssetId.",
       "writing_task"=>"writing_task: minWords=150 for Task1, 250 for Task2; prompt says Write at least ... words. No answer/options/slots.",
       _=>"speaking_prompt: English prompt, no answer/options/slots."
      });
    }
    let mut skeleton_questions=Vec::new();
    for item in &plan {
      let mut q=json!({"id":format!("q{}",item["number"]),"number":item["number"],"part":part,"kind":item["kind"],"prompt":"Complete English question instructions and context","points":item["points"],"evidence":["Exact continuous source quotation"]});
      let k=item["kind"].as_str().unwrap();
      if ["writing_task","speaking_prompt"].contains(&k){if k=="writing_task"{q["minWords"]=json!(if part==1{150}else{250});}}
      else {
        q["answer"]=json!({"accepted":["response"],"caseSensitive":false});
        if ["gap_fill","short_answer","diagram_label"].contains(&k){q["answer"]["maxWords"]=json!(if k=="diagram_label"{2}else{3});q["answer"]["maxNumbers"]=json!(1);}
        if ["single_choice","multiple_choice","matching","heading_match"].contains(&k){q["options"]=json!(["A. first option","B. second option","C. third option"]);q["answer"]["accepted"]=json!(["A. first option"]);}
        if k=="multiple_choice"{q["options"]=json!(["A. first","B. second","C. third","D. fourth","E. fifth"]);q["answer"]["accepted"]=json!([q["options"].as_array().unwrap()[..item["points"].as_u64().unwrap() as usize].to_vec()]);}
        if k=="matching"{q["slots"]=json!(["item to match"]);q["answer"]["accepted"]=json!([{"item to match":"A. first option"}]);}
        if k=="true_false_not_given"{q["answer"]["accepted"]=json!(["TRUE"]);}
        if k=="yes_no_not_given"{q["answer"]["accepted"]=json!(["YES"]);}
        if k=="diagram_label"{q["labels"]=json!([{"id":format!("label{}",item["number"]),"x":0.3,"y":0.5}]);q["answer"]["accepted"]=json!([{format!("label{}",item["number"]):"response"}]);}
      }
      if !["writing_task","speaking_prompt"].contains(&k){q.as_object_mut().unwrap().remove("evidence");q["evidenceSourceIds"]=json!([if section=="listening"{"t2"}else{"p1"}]);}
      skeleton_questions.push(q);
    }
    let mut skeleton=json!({"paragraphs":if section=="reading"{json!([{"id":"p1","part":part,"text":"Complete academic passage"}])}else{json!([])},"questions":skeleton_questions,"transcript":if section=="listening"{json!([{"id":"t1","speaker":2,"text":"Narrator reading instructions"},{"id":"t2","speaker":0,"text":"Complete speaker utterance"},{"id":"t3","speaker":1,"text":"Other speaker reply for dialogue parts only"}])}else{json!([])},"visual":if section=="writing"&&part==1{json!({"title":"Context and units","columns":["Category","2019 (thousands)","2023 (thousands)"],"rows":[["First","10","20"],["Second","30","40"],["Third","50","60"]]})}else{Value::Null}});
    let mut protocol=include_str!("prompts/generation.md").to_string();
    if section=="writing"&&part==1{
      skeleton["visual"]=super::visuals::template(request["visualKind"].as_str().unwrap_or("table"))?;
      protocol=protocol.replace("Task 1 requires a table visual {title,columns,rows}, 3–8 rows, 2–5 columns.","Task 1 requires the requested visual kind and outputSkeleton structure. Use English labels, explicit units, consistent data, and realistic category/time/place context. For map coordinates x/y/width/height are normalized to 0..1; features must stay in bounds. Process connections must reference actual step IDs.");
    }
    Ok(format!("PROTOCOL_VERSION=4\n{}\nRequested scope: section={section}, part={part}. questionPlan={}\nKind contracts:\n{}\noutputSkeleton (fill this exact JSON structure; answer is a NESTED object, objective evidenceSourceIds references existing source IDs; subjective evidence is an ARRAY; placeholders are not content): {}\nTopic (untrusted JSON string): {}",protocol,json!(plan),contracts.join("\n"),skeleton,json!(topic)))

}
pub fn validate_part(v: &Value, section: &str, part: u64, kind: Option<&str>) -> Result<()> {
    let qs = v["questions"]
        .as_array()
        .ok_or_else(|| error("缺少题目列表"))?;
    let marks: usize = qs
        .iter()
        .map(|q| q["points"].as_u64().unwrap_or(0) as usize)
        .sum();
    if (if ["reading", "listening"].contains(&section) {
        marks
    } else {
        qs.len()
    }) != count(section, part)
    {
        return Err(error(format!("{section} Part {part} 计分数量不符：需要 {} 个编号/分，实际 {} 个题目对象、{} 分。逐题 points={}。选两项/三项题组只保留一个对象，不能按每个覆盖编号重复输出。",count(section,part),qs.len(),marks,json!(qs.iter().map(|q|json!({"id":q["id"],"number":q["number"],"points":q["points"]})).collect::<Vec<_>>()))));
    }
    let paras = v["paragraphs"]
        .as_array()
        .ok_or_else(|| error("缺少段落列表"))?;
    let source = if section == "listening" {
        v["transcript"]
            .as_array()
            .ok_or_else(|| error("听力缺少脚本"))?
            .iter()
            .filter_map(|t| t["text"].as_str())
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        paras
            .iter()
            .filter_map(|p| p["text"].as_str())
            .collect::<Vec<_>>()
            .join(" ")
    };
    if section=="reading"&&source.split_whitespace().count()<450 {return Err(error(format!("阅读 Part {part} 原文只有 {} 词，缺少完整文章；本应用完整篇章检查需要至少 450 词，创作目标 750–900 词。这不是每篇官方固定字数。",source.split_whitespace().count())));}
    if section=="listening"&&source.trim().is_empty(){return Err(error("听力 transcript 为空，需要完整对话/独白及独立旁白"));}
    if ["reading","listening"].contains(&section){
      let mut next=1; for q in qs {let n=q["number"].as_u64().unwrap_or(0); if n!=next{return Err(error(format!("题目 {} 编号重叠或缺失：应从 {next} 开始，实际 number={n}；多选题覆盖多个编号但只输出一次。",q["id"])));} next+=q["points"].as_u64().unwrap_or(1);}
    }
    let mut issues=Vec::new();
    for q in qs { let result=(||->Result<()> {
        prompt(section, part, q["kind"].as_str(), "")?;
        if ["writing", "speaking"].contains(&section) && !q["answer"].is_null() {
            return Err(error("主观题不可带伪造的客观标准答案"));
        }
        if q["part"] != part || kind.is_some_and(|k| q["kind"] != k) {
            return Err(error("生成内容偏离所选部分/题型"));
        }
        if q["kind"]=="matching"&&q["slots"].as_array().is_none_or(|a|a.is_empty()) {return Err(error(format!("题目 {} 标成 matching，但缺少 slots 匹配项目；若为单项选择，kind 应为 single_choice。",q["id"])));}
        if ["reading", "listening"].contains(&section) {
            let pts = q["points"]
                .as_u64()
                .filter(|n| (1..=3).contains(n))
                .ok_or_else(|| error("每个编号一分，选2/3项组合题须声明2/3分"))?;
            if (pts > 1 && q["kind"] != "multiple_choice")
                || q["answer"]["accepted"]
                    .as_array()
                    .is_none_or(|a| a.is_empty())
            {
                return Err(error("客观题必须有标准答案；组合分数仅用于编号多选题"));
            }
            if q["kind"] == "multiple_choice"
                && q["answer"]["accepted"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|a| a.as_array().is_none_or(|a| a.len() != pts as usize))
            {
                return Err(error("多选题 accepted 中每个答案集合长度须等于计分编号数"));
            }
            let evidence = q["evidence"]
                .as_array()
                .ok_or_else(|| error("缺少答案依据"))?;
            if evidence.is_empty()
                || evidence.iter().any(|e| {
                    e.as_str()
                        .is_none_or(|s| s.trim().len() < 5 || !source.contains(s))
                })
            {
                return Err(error(format!("题目 {} evidence 中以下引文不在原文/脚本中：{}。请使用 evidenceSourceIds 指向该答案所在的段落/发言 ID，程序将提取真实原文；不要重新抄写引文。",q["id"],json!(evidence.iter().filter(|e|e.as_str().is_none_or(|s|!source.contains(s))).collect::<Vec<_>>()))));
            }
            if ["gap_fill", "short_answer"].contains(&q["kind"].as_str().unwrap_or(""))
                && q["answer"]["maxWords"].as_u64().is_none()
            {
                return Err(error(format!("题目 {} 填空/简答缺少明确字数上限 maxWords，需要与题干限制一致。",q["id"])));
            }
            if ["gap_fill", "short_answer"].contains(&q["kind"].as_str().unwrap_or("")) {
                for answer in q["answer"]["accepted"].as_array().unwrap().iter().filter_map(Value::as_str) {
                    let words=answer.split_whitespace().filter(|s|s.chars().any(char::is_alphabetic)).count() as u64;
                    if words>q["answer"]["maxWords"].as_u64().unwrap_or(u64::MAX) {
                        return Err(error(format!("题目 {} 答案 {:?} 有 {words} 个词，超过题干字数上限 {}；请修正题干/答案一致性，不要只补元数据。",q["id"],answer,q["answer"]["maxWords"])));
                    }
                }
            }
        }
        Ok(())
        })();if let Err(e)=result{issues.push(e.message);}
    }
    if !issues.is_empty(){return Err(error(issues.join("\n")));}
    if section == "listening" {
        let ts = v["transcript"].as_array().unwrap();
        if ts.iter().any(|t| {
            t["text"]
                .as_str()
                .is_none_or(|s| s.is_empty() || s.chars().count() > 500)
                || t["speaker"].as_u64().is_none_or(|n| n > 2)
        }) {
            return Err(error("听力脚本长度或说话人无效"));
        }
        let speakers: std::collections::HashSet<_> =
            ts.iter().map(|t| t["speaker"].as_u64().unwrap()).filter(|n|*n!=2).collect();
        if !ts.iter().any(|t|t["speaker"]==2){return Err(error("听力需要独立旁白角色，不能混用对话音色"));}
        if speakers.len() != if part == 1 || part == 3 { 2 } else { 1 } {
            return Err(error("听力说话人数与部分不符"));
        }
    }
    if section == "writing" && qs[0]["minWords"] != if part == 1 { 150 } else { 250 } {
        return Err(error("写作字数要求不正确"));
    }
    if section=="writing"&&part==1{super::visuals::render(&v["visual"])?;}
    if qs.iter().any(|q|q["kind"]=="diagram_label"){diagram_png(&v["visual"])?;}
    let mut local_questions=qs.clone();let mut local_assets=Vec::new();
    for q in &mut local_questions {if q["kind"]=="diagram_label"||section=="writing"&&part==1 {
        q["imageAssetId"]=json!("local-visual");if local_assets.is_empty(){local_assets.push(json!({"id":"local-visual","path":"media/local.png","sha256":"0".repeat(64),"mediaType":"image/png"}));}
    }}
    let paper=json!({"schemaVersion":1,"id":"text-contract-check","version":1,"title":"Text contract check","testType":"academic","status":"review_required","provenance":{"kind":"ai_generated","source":"unpublished structural validation"},"assets":local_assets,"sections":[{"section":section,"title":"Text check","durationMs":1000,"timingSource":"unpublished","paragraphs":paras,"questions":local_questions}]});
    crate::papers::validate(&paper)?;
    Ok(())
}

/// Original generation only: preserve source and answers, compile visible word limits consistently.
pub fn reconcile_generated_limits(v:&mut Value)->Result<()> {
 let re=regex::Regex::new(r"(?i)\b(?:NO\s+MORE\s+THAN\s+)?(ONE|TWO|THREE)\s+WORDS?(?:\s+ONLY)?\b").unwrap();
 for q in v["questions"].as_array_mut().ok_or_else(||error("缺少题目列表"))? {
  if !["gap_fill","short_answer"].contains(&q["kind"].as_str().unwrap_or("")){continue;}
  let needed=q["answer"]["accepted"].as_array().into_iter().flatten().filter_map(Value::as_str).map(|s|s.split_whitespace().filter(|s|s.chars().any(char::is_alphabetic)).count()).max().unwrap_or(0);
  if needed>3{return Err(error(format!("原创题 {} 的答案超过本蓝图三词上限，请缩短答案或调整设问；原文和答案未被程序改写",q["id"])));}
  let prompt=q["prompt"].as_str().unwrap_or("").to_owned();if let Some(caps)=re.captures(&prompt){let old=match caps[1].to_uppercase().as_str(){"ONE"=>1,"TWO"=>2,_=>3};let limit=old.max(needed);if limit>old{let new=match limit{1=>"ONE",2=>"TWO",_=>"THREE"};q["prompt"]=json!(re.replace_all(&prompt,format!("NO MORE THAN {new} WORDS")).to_string());}q["answer"]["maxWords"]=json!(limit);}
 }Ok(())
}
pub fn normalize_generated(v:&mut Value,section:&str,part:u64)->Result<()>{
 normalize_metadata(v,section,part)?;
 // A legacy model's retyped quote is never treated as source. Locate the actual
 // source only when all literal answers identify exactly one existing passage/turn.
 let sources:Vec<_>=["paragraphs","transcript"].iter().flat_map(|field|v[*field].as_array().into_iter().flatten()).filter(|s|s["speaker"]!=2).filter_map(|s|Some((s["id"].as_str()?.to_owned(),s["text"].as_str()?.to_owned()))).collect();
 for q in v["questions"].as_array_mut().ok_or_else(||error("缺少题目列表"))?{
  if !["gap_fill","short_answer"].contains(&q["kind"].as_str().unwrap_or("")){continue;}
  let bad_quote=q["evidence"].as_array().is_some_and(|es|es.iter().any(|e|e.as_str().is_some_and(|quote|!sources.iter().any(|(_,text)|text.contains(quote)))));
  if !bad_quote{continue;}let Some(answers)=q["answer"]["accepted"].as_array().filter(|a|!a.is_empty()&&a.iter().all(|a|a.as_str().is_some_and(|s|s.trim().chars().count()>=2)))else{continue;};
  let patterns:Vec<_>=answers.iter().map(|a|regex::Regex::new(&format!(r"(?i)(?:^|[^\p{{L}}\p{{N}}]){}(?:$|[^\p{{L}}\p{{N}}])",regex::escape(a.as_str().unwrap().trim())))).collect::<std::result::Result<_,_>>().map_err(error)?;
  let matches:Vec<_>=sources.iter().filter(|(_,text)|patterns.iter().all(|p|p.is_match(text))).collect();if matches.len()==1{q["evidenceSourceIds"]=json!([matches[0].0]);}
 }
 normalize_metadata(v,section,part)?;reconcile_generated_limits(v)
}
pub fn normalize_metadata(v: &mut Value, section: &str, part: u64) -> Result<()> {
    let mut source_map=std::collections::BTreeMap::new();
    for (field,prefix) in [("paragraphs","p"),("transcript","t")]{
        if let Some(items)=v[field].as_array_mut(){for (i,item) in items.iter_mut().enumerate(){
            if item["id"].is_null(){item["id"]=json!(format!("{prefix}{}",i+1));}
            if field=="paragraphs"&&item["part"].is_null(){item["part"]=json!(part);}
            let id=item["id"].as_str().ok_or_else(||error("来源段落/发言 id 必须是字符串"))?.to_owned();
            let text=item["text"].as_str().ok_or_else(||error("来源段落/发言缺少 text"))?.to_owned();
            if source_map.insert(id.clone(),(text,item["speaker"]==2)).is_some(){return Err(error(format!("来源 ID {id} 重复，证据无法唯一绑定")));}
        }}
    }
    let mut source_refs=v["evidenceSourceRefs"].as_object().cloned().unwrap_or_default();
    if let Some(qs)=v["questions"].as_array_mut(){
      let mut normalized:Vec<Value>=Vec::new();
      for mut q in std::mem::take(qs){
        if !q.is_object(){return Err(error("questions 中每项必须是题目对象"));}
        if let Some(refs)=q.get("evidenceSourceIds").cloned(){
            let ids=refs.as_array().filter(|a|!a.is_empty()&&a.len()<=6).ok_or_else(||error(format!("题目 {} evidenceSourceIds 需有 1–6 个来源 ID",q["id"])))?;
            let mut evidence=Vec::new();for id in ids {
                let id=id.as_str().ok_or_else(||error("证据来源 ID 必须为字符串"))?;
                let (text,narrator)=source_map.get(id).ok_or_else(||error(format!("题目 {} 引用了不存在的来源 {id}；可用 ID：{}",q["id"],source_map.keys().cloned().collect::<Vec<_>>().join(","))))?;
                if section=="listening"&&*narrator{return Err(error(format!("题目 {} 引用了旁白 {id}，答案依据必须来自实际对话/独白",q["id"])));}
                evidence.push(text.clone());
            }
            source_refs.insert(q["id"].as_str().unwrap_or("unknown").to_owned(),refs);q["evidence"]=json!(evidence);q.as_object_mut().unwrap().remove("evidenceSourceIds");
        }
        if q["answer"].is_null()&&q["accepted"].is_array(){q["answer"]=json!({});}
        if q["answer"].is_object(){for field in ["accepted","caseSensitive","maxWords","maxNumbers"]{
            if let Some(value)=q.get(field).cloned(){if !q["answer"][field].is_null()&&q["answer"][field]!=value{return Err(error(format!("题目 {} 的 {field} 在题目与 answer 内相互矛盾",q["id"])));}q["answer"][field]=value;q.as_object_mut().unwrap().remove(field);}
        }}
          if ["matching","diagram_label"].contains(&q["kind"].as_str().unwrap_or(""))&&q["answer"]["accepted"].is_object(){q["answer"]["accepted"]=json!([q["answer"]["accepted"].clone()]);}
          if let Some(e)=q["evidence"].as_str().map(str::to_owned){q["evidence"]=json!([e]);}

        if q["kind"]=="multiple_choice"&&q["points"].as_u64().is_some_and(|n|n>1){
          if let Some(a)=q["answer"]["accepted"].as_array().filter(|a|a.iter().all(Value::is_string)).cloned(){q["answer"]["accepted"]=json!([a]);}
          let n=q["number"].as_u64().unwrap_or(0);
          let duplicate=normalized.iter().any(|prior|{
            let start=prior["number"].as_u64().unwrap_or(0);let width=prior["points"].as_u64().unwrap_or(0);
            if prior["kind"]!="multiple_choice"||n<=start||n>=start+width{return false;}
            let mut a=prior.clone();let mut b=q.clone();for key in ["id","number"]{a.as_object_mut().unwrap().remove(key);b.as_object_mut().unwrap().remove(key);}a==b
          });if duplicate{continue;}
        }
        normalized.push(q);
      } *qs=normalized;
    }
    for q in v["questions"]
        .as_array_mut()
        .ok_or_else(|| error("缺少题目"))?
    {
        if !q.is_object() {
            return Err(error("每道题目必须是JSON对象"));
        }
        if q["answer"]["accepted"].is_array() {
            if q["answer"]["caseSensitive"].is_null(){q["answer"]["caseSensitive"]=json!(false);}
            if ["gap_fill","short_answer"].contains(&q["kind"].as_str().unwrap_or(""))&&q["answer"]["maxWords"].is_null(){
                let text=q["prompt"].as_str().unwrap_or("").to_uppercase();
                let re=regex::Regex::new(r"\b(ONE|TWO|THREE|FOUR|FIVE|[1-9])\s+WORDS?\b").unwrap();
                if let Some(c)=re.captures(&text){let limit=match &c[1]{"ONE"=>1,"TWO"=>2,"THREE"=>3,"FOUR"=>4,"FIVE"=>5,n=>n.parse::<u64>().unwrap()};q["answer"]["maxWords"]=json!(limit);}
                if text.contains("A NUMBER"){q["answer"]["maxNumbers"]=json!(1);}
            }
        }
        if q["kind"]=="matching"&&q["slots"].is_null()&&q["prompt"].as_str().is_some_and(|s|s.to_uppercase().contains("CHOOSE ONE"))&&q["answer"]["accepted"].as_array().is_some_and(|a|!a.is_empty()&&a.iter().all(|v|v.is_string()&&q["options"].as_array().is_some_and(|opts|opts.contains(v)))) {
            q["kind"]=json!("single_choice");
        }
        for name in ["maxWords", "maxNumbers"] {
            if let Some(value) = q.get(name).cloned() {
                if !q["answer"].is_object() {
                    return Err(error("字数约束缺少对应的标准答案对象"));
                }
                if !q["answer"][name].is_null() && q["answer"][name] != value {
                    return Err(error("生成的字数约束相互矛盾"));
                }
                q["answer"][name] = value;
                q.as_object_mut().unwrap().remove(name);
            }
        }
    }
    v["evidenceSourceRefs"]=Value::Object(source_refs);
    if section == "writing" {
        for q in v["questions"]
            .as_array_mut()
            .ok_or_else(|| error("缺少题目"))?
        {
            let words = if part == 1 { 150 } else { 250 };
            if q["prompt"]
                .as_str()
                .is_none_or(|s| !s.contains(&words.to_string()))
            {
                return Err(error("写作题干缺少正确的最低字数要求"));
            }
            if q["answer"]["accepted"]
                .as_array()
                .is_some_and(|a| a.is_empty())
            {
                q.as_object_mut().unwrap().remove("answer");
            }
            q["minWords"] = json!(words);
        }
    }
    Ok(())
}
pub fn table_png(v: &Value) -> Result<Vec<u8>> {
    use font8x8::UnicodeFonts;
    let title = v["title"]
        .as_str()
        .ok_or_else(|| error("Task1 缺少表格标题"))?;
    let cols = v["columns"]
        .as_array()
        .ok_or_else(|| error("Task1 缺少列名"))?;
    let rows = v["rows"]
        .as_array()
        .ok_or_else(|| error("Task1 缺少表格数据"))?;
    if !(2..=5).contains(&cols.len())
        || !(3..=8).contains(&rows.len())
        || title.chars().count() > 120
    {
        return Err(error("Task1 表格尺寸或标题无效"));
    }
    fn display(s: &str) -> Result<String> {
        let s = s
            .replace(['–', '—', '−'], "-")
            .replace(['‘', '’'], "'")
            .replace(['“', '”'], "\"");
        if s.chars().any(|c| {
            font8x8::BASIC_FONTS
                .get(c)
                .or_else(|| font8x8::LATIN_FONTS.get(c))
                .is_none()
        }) {
            return Err(error(
                "表格包含当前字库不支持的字符，请使用英文和常用数字单位",
            ));
        }
        Ok(s)
    }
    fn wrap(s: &str, width: usize) -> Vec<String> {
        let mut out = Vec::new();
        let mut line = String::new();
        for word in s.split_whitespace() {
            if line.chars().count() + word.chars().count() + 1 > width && !line.is_empty() {
                out.push(std::mem::take(&mut line));
            }
            let chars: Vec<char> = word.chars().collect();
            for chunk in chars.chunks(width) {
                let word: String = chunk.iter().collect();
                if line.chars().count() + word.chars().count() + 1 > width && !line.is_empty() {
                    out.push(std::mem::take(&mut line));
                }
                if !line.is_empty() {
                    line.push(' ');
                }
                line.push_str(&word);
            }
        }
        out.push(line);
        out
    }
    let title = display(title)?;
    let mut lines = wrap(&title, cols.len() * 24);
    lines.push(String::new());
    for row in std::iter::once(cols).chain(
        rows.iter()
            .map(|r| r.as_array())
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| error("表格行格式错误"))?
            .into_iter(),
    ) {
        if row.len() != cols.len() {
            return Err(error("表格列数不一致"));
        }
        let mut cells = Vec::new();
        for cell in row {
            let s = cell.as_str().ok_or_else(|| error("表格单元格应为字符串"))?;
            if s.chars().count() > 120 {
                return Err(error("表格单元格过长"));
            }
            cells.push(wrap(&display(s)?, 20));
        }
        for line_i in 0..cells.iter().map(|c| c.len()).max().unwrap_or(1) {
            let mut line = String::new();
            for c in &cells {
                line.push_str(&format!(
                    "{:<22}",
                    c.get(line_i).map(String::as_str).unwrap_or("")
                ));
            }
            lines.push(line);
        }
        lines.push("-".repeat(cols.len() * 22));
    }
    let width = (lines.iter().map(|s| s.chars().count()).max().unwrap_or(1) * 16 + 48) as u32;
    let height = (lines.len() * 30 + 48) as u32;
    let mut pixels = vec![250u8; (width * height * 3) as usize];
    for (line_i, line) in lines.iter().enumerate() {
        for (ci, ch) in line.chars().enumerate() {
            if let Some(glyph) = font8x8::BASIC_FONTS
                .get(ch)
                .or_else(|| font8x8::LATIN_FONTS.get(ch))
            {
                for (y, bits) in glyph.iter().enumerate() {
                    for x in 0..8 {
                        if bits & (1 << x) != 0 {
                            for dy in 0..2 {
                                for dx in 0..2 {
                                    let px = 24 + ci * 16 + x * 2 + dx;
                                    let py = 24 + line_i * 30 + y * 2 + dy;
                                    let i = (py * width as usize + px) * 3;
                                    if i + 2 < pixels.len() {
                                        pixels[i..i + 3].fill(35);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, width, height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut w = encoder.write_header().map_err(error)?;
        w.write_image_data(&pixels).map_err(error)?;
    }
    Ok(bytes)
}
pub fn diagram_png(v: &Value) -> Result<Vec<u8>> {
    use font8x8::UnicodeFonts;
    let items = v["items"]
        .as_array()
        .ok_or_else(|| error("标注图缺少节点"))?;
    if !(2..=35).contains(&items.len()) {
        return Err(error("标注图需2至35个节点"));
    }
    let w = 1200usize;
    let h = 800usize;
    let mut pixels = vec![250u8; w * h * 3];
    fn dot(p: &mut [u8], x: i32, y: i32) {
        if (0..1200).contains(&x) && (0..800).contains(&y) {
            let i = (y as usize * 1200 + x as usize) * 3;
            p[i..i + 3].fill(45);
        }
    }
    fn line(p: &mut [u8], a: (i32, i32), b: (i32, i32)) {
        let steps = (b.0 - a.0).abs().max((b.1 - a.1).abs()).max(1);
        for t in 0..=steps {
            dot(
                p,
                a.0 + (b.0 - a.0) * t / steps,
                a.1 + (b.1 - a.1) * t / steps,
            );
        }
    }
    let mut nodes = std::collections::HashMap::new();
    for n in items {
        let id = n["id"].as_str().ok_or_else(|| error("图示节点缺少 ID"))?;
        let x = n["x"]
            .as_f64()
            .filter(|x| (0.15..=0.85).contains(x))
            .ok_or_else(|| error("图示横坐标越界"))?;
        let y = n["y"]
            .as_f64()
            .filter(|y| (0.15..=0.85).contains(y))
            .ok_or_else(|| error("图示纵坐标越界"))?;
        if nodes
            .insert(id, ((x * w as f64) as i32, (y * h as f64) as i32))
            .is_some()
        {
            return Err(error("图示 ID 重复"));
        }
    }
    for c in v["connections"].as_array().into_iter().flatten() {
        let a = nodes
            .get(c["from"].as_str().unwrap_or(""))
            .ok_or_else(|| error("图示连接起点不存在"))?;
        let b = nodes
            .get(c["to"].as_str().unwrap_or(""))
            .ok_or_else(|| error("图示连接终点不存在"))?;
        line(&mut pixels, *a, *b);
    }
    for n in items {
        let (x, y) = nodes[n["id"].as_str().unwrap()];
        let text = n["text"]
            .as_str()
            .filter(|s| s.is_ascii() && s.len() <= 24)
            .ok_or_else(|| error("节点文字须为24字符内英文"))?;
        for py in y - 25..=y + 25 {
            for px in x - 110..=x + 110 {
                let i = (py as usize * w + px as usize) * 3;
                pixels[i..i + 3].fill(250);
            }
        }
        for (a, b) in [
            ((x - 110, y - 25), (x + 110, y - 25)),
            ((x - 110, y + 25), (x + 110, y + 25)),
            ((x - 110, y - 25), (x - 110, y + 25)),
            ((x + 110, y - 25), (x + 110, y + 25)),
        ] {
            line(&mut pixels, a, b);
        }
        for (ci, ch) in text.chars().enumerate() {
            if let Some(glyph) = font8x8::BASIC_FONTS.get(ch) {
                for (gy, bits) in glyph.iter().enumerate() {
                    for gx in 0..8 {
                        if bits & (1 << gx) != 0 {
                            dot(
                                &mut pixels,
                                x - text.len() as i32 * 4 + ci as i32 * 8 + gx,
                                y - 4 + gy as i32,
                            );
                        }
                    }
                }
            }
        }
    }
    let title = v["title"].as_str().unwrap_or("Diagram");
    if !title.is_ascii() || title.len() > 100 {
        return Err(error("图示标题无效"));
    }
    for (ci, ch) in title.chars().enumerate() {
        if let Some(glyph) = font8x8::BASIC_FONTS.get(ch) {
            for (y, bits) in glyph.iter().enumerate() {
                for x in 0..8 {
                    if bits & (1 << x) != 0 {
                        dot(&mut pixels, 24 + ci as i32 * 8 + x, 24 + y as i32);
                    }
                }
            }
        }
    }
    let mut bytes = Vec::new();
    {
        let mut e = png::Encoder::new(&mut bytes, w as u32, h as u32);
        e.set_color(png::ColorType::Rgb);
        e.set_depth(png::BitDepth::Eight);
        let mut writer = e.write_header().map_err(error)?;
        writer.write_image_data(&pixels).map_err(error)?;
    }
    Ok(bytes)
}

pub fn stages(req: &Value) -> Result<Vec<(String, u64)>> {
    let sections = if req["section"] == "all" {
        vec!["listening", "reading", "writing", "speaking"]
    } else {
        vec![req["section"].as_str().ok_or_else(|| error("请选择科目"))?]
    };
    let mut out = Vec::new();
    for s in sections {
        let max = match s {
            "listening" => 4,
            "reading" | "speaking" => 3,
            "writing" => 2,
            _ => return Err(error("科目无效")),
        };
        for p in 1..=max {
            if req["part"].as_u64().is_none_or(|n| n == p) || req["section"] == "all" {
                prompt(
                    s,
                    p,
                    req["kind"].as_str(),
                    req["topic"].as_str().unwrap_or(""),
                )?;
                out.push((s.into(), p));
            }
        }
    }
    if out.is_empty() {
        return Err(error("部分选择无效"));
    }
    Ok(out)
}
