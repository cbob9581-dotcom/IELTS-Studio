import {test,expect} from 'vitest';
import {analyzeArticulation} from './articulation';
test('mouth rounding responds to actual acoustic shape, stays bounded and closes in silence',()=>{
 const signal=(frequency:number)=>Float32Array.from({length:800},(_,i)=>Math.sin(2*Math.PI*frequency*i/24000)*.2);
 const low=analyzeArticulation(signal(500),24000,0),high=analyzeArticulation(signal(3000),24000,0),quiet=analyzeArticulation(new Float32Array(800),24000,0);
 expect(low.rounding).toBeGreaterThan(high.rounding);expect(low.energy).toBeGreaterThan(0);expect(low.energy).toBeLessThanOrEqual(1);expect(quiet).toEqual({energy:0,rounding:0,width:0});
});
