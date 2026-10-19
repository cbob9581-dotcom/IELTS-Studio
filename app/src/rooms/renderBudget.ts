export function shouldDraw(now:number,last:number,visible:boolean,interactive:boolean,animated:boolean){
 if(!visible)return false;
 return interactive||now-last>=1000/(animated?30:15);
}
