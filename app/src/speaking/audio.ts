import type { AppConfig } from "../configuration/types";
import type { MutableRefObject } from "react";
export function encodeWave(chunks:Float32Array[],rate:number):ArrayBuffer {
  const length=chunks.reduce((n,c)=>n+c.length,0),bytes=new ArrayBuffer(44+length*2),v=new DataView(bytes);
  const str=(o:number,t:string)=>[...t].forEach((c,i)=>v.setUint8(o+i,c.charCodeAt(0)));
  str(0,"RIFF");v.setUint32(4,36+length*2,true);str(8,"WAVE");str(12,"fmt ");v.setUint32(16,16,true);
  v.setUint16(20,1,true);v.setUint16(22,1,true);v.setUint32(24,rate,true);v.setUint32(28,rate*2,true);
  v.setUint16(32,2,true);v.setUint16(34,16,true);str(36,"data");v.setUint32(40,length*2,true);
  let offset=44;for(const chunk of chunks)for(const sample of chunk){const s=Math.max(-1,Math.min(1,sample));v.setInt16(offset,Math.round(s*(s<0?32768:32767)),true);offset+=2;}
  return bytes;
}
export function base64Wave(buffer:ArrayBuffer):string {
  const bytes=new Uint8Array(buffer);let s="";
  for(let i=0;i<bytes.length;i+=8192)s+=String.fromCharCode(...bytes.subarray(i,i+8192));
  return btoa(s);
}
export function splitUtterance(text:string):string[]{
 const chunks:string[]=[];let line="";
 for(let word of text.trim().split(/\s+/)){
  if(line&&line.length+word.length+1>500){chunks.push(line);line="";}
  while(word.length>500){chunks.push(word.slice(0,500));word=word.slice(500);}
  if(word)line+=(line?" ":"")+word;
 }if(line)chunks.push(line);return chunks;
}
export class Microphone {
  private context:AudioContext;
  private node:AudioWorkletNode;
  private active=false;
  private chunks:Float32Array[]=[];
  private segment:Float32Array[]=[];
  private checkpoint=0;
  private segmentAt=0;
  private started=0;
  private lastVoice=0;
  private voicedMs=0;
  private options:{long:boolean;deadline:number;onDone:(wav:ArrayBuffer,voiced:boolean)=>void;onCheckpoint:(wav:ArrayBuffer,index:number)=>void}|null=null;
  private timer:ReturnType<typeof setInterval>|undefined;
  private source:MediaStreamAudioSourceNode;
  private sink:GainNode;
  private constructor(private stream:MediaStream,context:AudioContext,node:AudioWorkletNode){
    this.context=context;this.node=node;this.source=context.createMediaStreamSource(stream);
    this.sink=context.createGain();this.sink.gain.value=0;
    this.source.connect(node).connect(this.sink).connect(context.destination);
    node.port.onmessage=e=>{
      if(!this.active)return;
      const samples=e.data as Float32Array;
      this.chunks.push(samples);this.segment.push(samples);
      let sum=0;for(const x of samples)sum+=x*x;
      if(Math.sqrt(sum/samples.length)>.012){this.lastVoice=Date.now();this.voicedMs+=samples.length/context.sampleRate*1000;}
    };
  }
  static async open(deviceId:string,rate:number):Promise<Microphone>{
    const stream=await navigator.mediaDevices.getUserMedia({audio:{deviceId:deviceId?{exact:deviceId}:undefined,channelCount:1,echoCancellation:true,noiseSuppression:true},video:false});
    let context:AudioContext|undefined;
    try{
      context=new AudioContext({sampleRate:rate});
      await context.audioWorklet.addModule(new URL("/audio-capture.js",document.baseURI).href);
      await context.resume();return new Microphone(stream,context,new AudioWorkletNode(context,"ielts-capture"));
    }catch(e){stream.getTracks().forEach(t=>t.stop());await context?.close();throw e;}
  }
  begin(options:NonNullable<Microphone["options"]>){
    this.active=true;this.options=options;this.chunks=[];this.segment=[];this.checkpoint=0;
    this.started=Date.now();this.segmentAt=this.started;this.lastVoice=this.started;this.voicedMs=0;
    clearInterval(this.timer);
    this.timer=setInterval(()=>{
      const now=Date.now(),elapsed=now-this.started;
      if(now-this.segmentAt>=10000&&this.segment.length){
        options.onCheckpoint(encodeWave(this.segment,this.context.sampleRate),this.checkpoint++);
        this.segment=[];this.segmentAt=now;
      }
      const silence=now-this.lastVoice;
      const naturalEnd=this.voicedMs>350&&silence>=(options.long?10000:8000)&&elapsed>=(options.long?60000:1500);
      if(now>=options.deadline||naturalEnd){const voiced=this.voicedMs>350;const wav=this.stop();options.onDone(wav,voiced);}
    },100);
  }
  suspend():{wav:ArrayBuffer;voiced:boolean}|null {
    if(!this.active)return null;
    const voiced=this.voicedMs>350;return {wav:this.stop(),voiced};
  }
  stop():ArrayBuffer {
    this.active=false;clearInterval(this.timer);this.timer=undefined;
    return encodeWave(this.chunks,this.context.sampleRate);
  }
  async dispose(){this.stop();this.node.port.onmessage=null;this.node.disconnect();this.source.disconnect();this.sink.disconnect();this.stream.getTracks().forEach(t=>t.stop());await this.context.close();}
}
export async function playSpeech(bytes:ArrayBuffer,config:AppConfig,signal:AbortSignal,level:MutableRefObject<number>):Promise<void>{
  signal.throwIfAborted();const url=URL.createObjectURL(new Blob([bytes],{type:"audio/wav"}));
  const audio=new Audio(url);audio.volume=config.audio.masterVolume*config.audio.ttsVolume;
  let context:AudioContext|undefined,pcm:Float32Array|undefined,rate=24000;
  let ticker:ReturnType<typeof setInterval>|undefined;
  try{
    if(config.audio.outputDeviceId){
      if(!("setSinkId" in audio))throw Error("当前播放环境不支持所选输出设备");
      await (audio as HTMLAudioElement &{setSinkId:(id:string)=>Promise<void>}).setSinkId(config.audio.outputDeviceId);
    }
    context=new AudioContext();
    const buffer=await context.decodeAudioData(bytes.slice(0));pcm=buffer.getChannelData(0);rate=buffer.sampleRate;
    await context.close();context=undefined;signal.throwIfAborted();
    await new Promise<void>((resolve,reject)=>{
      const abort=()=>{audio.pause();cleanup();reject(new DOMException("已取消","AbortError"));};
      const cleanup=()=>{signal.removeEventListener("abort",abort);audio.onended=null;audio.onerror=null;};
      audio.onended=()=>{cleanup();resolve();};audio.onerror=()=>{cleanup();reject(Error("考官音频无法播放"));};
      signal.addEventListener("abort",abort,{once:true});
      ticker=setInterval(()=>{
        const at=Math.floor(audio.currentTime*rate);let sum=0;
        for(let i=at;i<Math.min(at+800,pcm!.length);i++)sum+=pcm![i]*pcm![i];
        level.current=Math.min(1,Math.sqrt(sum/800)*6);
      },33);
      audio.play().catch(()=>{cleanup();reject(Error("考官语音播放被阻止，请点击重试并检查输出设备"));});
    });
  }finally{clearInterval(ticker);level.current=0;audio.pause();audio.src="";URL.revokeObjectURL(url);await context?.close();}
}
