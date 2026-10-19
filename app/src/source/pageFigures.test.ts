import {it,expect} from 'vitest';
import {isQuestionFigure} from './pageFigures';
it('preserves unknown small diagrams and avoids vision for readable HTML tables',()=>{
 expect(isQuestionFigure({id:'b',page:1,type:'image',text:'',imageAssetId:'small',bbox:[.1,.1,.15,.15],auxiliaryImage:true})).toBe(true);
 expect(isQuestionFigure({id:'b',page:1,type:'table',text:'<table><tr><td>answer</td></tr></table>',imageAssetId:'table'})).toBe(false);
});
