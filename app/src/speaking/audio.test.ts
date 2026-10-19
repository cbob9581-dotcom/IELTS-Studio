import { expect,test } from "vitest";
import { encodeWave, splitUtterance, Microphone } from "./audio";
test("microphone samples remain mono PCM16 with matching WAV length and sample rate",()=>{
 const b=encodeWave([new Float32Array([-1,0,1])],24000);const v=new DataView(b);
 expect(String.fromCharCode(...new Uint8Array(b,0,4))).toBe("RIFF");
 expect(v.getUint32(24,true)).toBe(24000);expect(v.getUint16(22,true)).toBe(1);
 expect(v.getUint32(40,true)).toBe(6);expect(v.getInt16(44,true)).toBe(-32768);expect(v.getInt16(48,true)).toBe(32767);
});
test("TTS chunks keep all words inside provider limits",()=>{
 const text=Array.from({length:140},(_,i)=>`word${i}`).join(" ");
 const chunks=splitUtterance(text);expect(chunks.length).toBeGreaterThan(1);
 expect(chunks.every(x=>x.length<=500)).toBe(true);expect(chunks.join(" ")).toBe(text);
});

test("suspending a capture returns its full audio before dispose and cannot complete twice",()=>{
 const m=Object.create(Microphone.prototype) as Microphone;
 Object.assign(m,{context:{sampleRate:24000},active:true,chunks:[new Float32Array(240)],voicedMs:500});
 const capture=m.suspend();
 expect(capture?.wav.byteLength).toBe(524);expect(capture?.voiced).toBe(true);
 expect(m.suspend()).toBeNull();
});
