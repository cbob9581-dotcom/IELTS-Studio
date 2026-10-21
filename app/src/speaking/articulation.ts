/** Acoustic mouth-shape heuristic; this is not phoneme recognition or forced alignment. */
export type Articulation={energy:number;rounding:number;width:number};
export function analyzeArticulation(pcm:Float32Array,rate:number,at:number):Articulation{
 const end=Math.min(at+800,pcm.length),count=Math.max(0,end-at);if(!count||!Number.isFinite(rate)||rate<=0)return {energy:0,rounding:0,width:0};
 let sum=0;for(let i=at;i<end;i++)sum+=pcm[i]*pcm[i];const energy=Math.min(1,Math.sqrt(sum/count)*6);if(energy<.025)return {energy:0,rounding:0,width:0};
 function band(frequency:number){const coeff=2*Math.cos(2*Math.PI*frequency/rate);let a=0,b=0;for(let i=at;i<end;i++){const next=pcm[i]+coeff*a-b;b=a;a=next;}return Math.max(0,a*a+b*b-coeff*a*b);}
 const low=band(500),middle=band(1500),high=band(3000),total=low+middle+high;const rounding=total>1e-8?Math.max(0,Math.min(1,low/total)):0.35;
 return {energy,rounding,width:energy*(1-rounding)};
}
