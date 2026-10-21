export type TaskRequest={kind:'cloud'|'source';owner:string;targets:string[];title:string};
export type BackgroundTask=TaskRequest&{id:string;state:'queued'|'running'|'pausing'|'paused'|'ready'|'failed'|'cancelled';completed:string[];error?:string|null;createdAt:number;updatedAt:number;usage?:{requests:number;inputTokens:number|null;outputTokens:number|null};operation?:{label?:string;done?:number;total?:number;unit?:string;targetId?:string}|null};
export const taskActive=(state:string)=>['queued','running','pausing'].includes(state);
