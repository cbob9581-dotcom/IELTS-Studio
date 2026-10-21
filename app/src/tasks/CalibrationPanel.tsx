import {useEffect,useState} from 'react';
import type {DesktopPort} from '../platform/desktop';
import {errorText} from '../platform/desktop';
export type CalibrationSummary={notice:string;groups:{model:string;protocol:string;uniqueSamples:number;runs:number;meanAbsoluteError:number;withinHalfBand:number;repeatStandardDeviation:number|null;targetMet:boolean}[]};
export function CalibrationPanel({port}:{port:DesktopPort}){
 const [summary,setSummary]=useState<CalibrationSummary|null>(null),[error,setError]=useState(''),[busy,setBusy]=useState(false);
 useEffect(()=>{let alive=true;void port.calibrationRead?.().then(s=>{if(alive)setSummary(s);}).catch(e=>{if(alive)setError(errorText(e));});return()=>{alive=false;};},[port]);
 if(!port.calibrationImport)return null;
 return <details className="settings-card"><summary>评分基准检查（本地统计）</summary><p>用独立人工参考评分检查模型误差和重复评分波动。导入不会发送云请求，也不会自动修改评分结果。参考集与调参集应分开；目前没有预装经过人工标注的参考集。</p><p>UTF-8 JSON 数组或 JSONL，每条包含以下字段：</p><pre>{'{"id":"sample-001","referenceBand":6.5,"predictedBand":6,"model":"填写实际模型","protocol":"whole-session-v1","referenceSource":"人工标注来源"}'}</pre><button disabled={busy} onClick={async()=>{setBusy(true);setError('');try{const path=await port.chooseFile(['json','jsonl']);if(path)setSummary(await port.calibrationImport!(path));}catch(e){setError(errorText(e));}finally{setBusy(false);}}}>导入参考评分并统计</button>{error&&<p className="error" role="alert">{error}</p>}{summary&&<><p>{summary.notice}</p>{summary.groups.map(g=><p key={`${g.model}/${g.protocol}`}>{g.model} / {g.protocol}：{g.uniqueSamples} 个独立样本，{g.runs} 次评分；平均绝对误差 {g.meanAbsoluteError.toFixed(2)} 分；误差不超过半分 {(g.withinHalfBand*100).toFixed(1)}%；重复评分标准差 {g.repeatStandardDeviation?.toFixed(2)??'尚无重复样本'}。{g.targetMet?'达到本参考集统计目标。':'尚未达到参考集统计目标（至少100个独立样本、平均误差≤0.5、半分内比例≥80%）。'}</p>)}</>}</details>;
}
