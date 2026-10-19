import type {Block} from './types';
export function isQuestionFigure(b:Block):boolean{
 if(!b.imageAssetId||(b.auxiliaryImage===true&&b.auxiliaryConfirmed===true))return false;
 if(b.type==='table'&&b.text.includes('<table'))return false;
 return true;
}
