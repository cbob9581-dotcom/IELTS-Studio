import {useEffect,useState} from "react";
import type {DesktopPort} from "../platform/desktop";
import {providerLabels,providerIds} from "../configuration/types";
export function ServiceStatus({port,refreshKey}:{port:DesktopPort;refreshKey:string}){
 const [lines,setLines]=useState<string[]>([]);
 useEffect(()=>{let alive=true;
  Promise.all([port.getConfig(),port.credentialStatus()]).then(([config,keys])=>{if(alive)setLines(providerIds.map(id=>`${providerLabels[id]}：${!config.providers[id].enabled?"未启用":!keys[id]?"缺少密钥":!config.providers[id].baseUrl||!config.providers[id].model?"地址或模型未填完整":"配置已就绪，连通结果请看设置页的实际测试"}`));}).catch(()=>{if(alive)setLines(["配置读取失败，请打开设置检查。"]);});
  return()=>{alive=false;};
 },[port,refreshKey]);
 return <details><summary>查看当前服务配置状态</summary>{lines.map(line=><p key={line}>{line}</p>)}</details>;
}
