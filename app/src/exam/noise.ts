export type NoiseKind="handwriting"|"keyboard"|"waterPump";
export function noiseEvent(seed:string,index:number,enabled:NoiseKind[],profile:string){
 if(profile==="clean"||!enabled.length)return null;
 let h=2166136261;for(const c of `${seed}:${index}`)h=Math.imul(h^c.charCodeAt(0),16777619);
 const r=()=>{h^=h<<13;h^=h>>>17;h^=h<<5;return(h>>>0)/4294967296;};
 return{kind:enabled[Math.floor(r()*enabled.length)],delayMs:Math.floor((profile==="challenge"?4+r()*9:8+r()*16)*1000),seed:h>>>0};
}
export function playNoise(context:AudioContext,destination:AudioNode,kind:NoiseKind,seed:number){
 const length=kind==="waterPump"?3:kind==="handwriting"?1.7:.9;
 const buffer=context.createBuffer(1,Math.ceil(context.sampleRate*length),context.sampleRate),pcm=buffer.getChannelData(0);
 let n=seed||1;
 for(let i=0;i<pcm.length;i++){
  n^=n<<13;n^=n>>>17;n^=n<<5;const t=i/context.sampleRate,random=((n>>>0)/4294967296*2-1);
  const envelope=Math.min(1,t*20)*Math.min(1,(length-t)*10);
  pcm[i]=envelope*(kind==="waterPump"?.14*Math.sin(t*Math.PI*120)+random*.12:kind==="keyboard"?random*Math.exp(-(t%.13)*95)*.7:random*(.1+.16*Math.sin(t*32)**2));
 }
 const source=context.createBufferSource(),filter=context.createBiquadFilter();source.buffer=buffer;
 filter.type=kind==="handwriting"?"bandpass":"lowpass";filter.frequency.value=kind==="waterPump"?500:kind==="keyboard"?2800:1500;
 source.connect(filter).connect(destination);source.onended=()=>{source.disconnect();filter.disconnect();};source.start();
}
