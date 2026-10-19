import {render,fireEvent} from '@testing-library/react';
import {it,expect,vi} from 'vitest';
import {ListeningPlayer} from './ListeningPlayer';
it('starts selected range at marker and finishes at its end',()=>{
 const changed=vi.fn();const view=render(<ListeningPlayer urls={['audio.wav']} ranges={[{startMs:1000,endMs:3000}]} mode="training" paused={false} onReady={()=>{}} onError={()=>{}} onPosition={changed}/>);
 const a=view.container.querySelector('audio')!;Object.defineProperty(a,'duration',{value:10});fireEvent.loadedMetadata(a);expect(a.currentTime).toBe(1);a.pause=vi.fn();a.currentTime=3;fireEvent.timeUpdate(a);expect(changed).toHaveBeenLastCalledWith(expect.objectContaining({finished:true}));
});
