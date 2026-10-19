export type CapabilityStatus = {
  name: string;
  status: "unconfigured" | "available" | "error" | "adapter_ready";
  reason: string;
};
export function getCapabilities(): CapabilityStatus[] {
  return [
    {name:"MinerU 资料解析",status:"adapter_ready",reason:"上传、批次轮询、解压、AI 清洗与原文核对；需本地配置和真实文件验证"},
    {name:"写作 / 口语评测",status:"adapter_ready",reason:"四项反馈、录音证据和本地报告；模型分数为未校准参考估分"},
    {name:"考场随机噪音",status:"available",reason:"本地环境声调度已接入，设置中选择轻度或挑战；安静模式关闭"},
    {name:"Qwen ASR / TTS",status:"adapter_ready",reason:"已提供口语 REST 适配器；需填写配置、密钥并通过实际云端请求验证"},
    {name:"DeepSeek 出题 / 考官",status:"adapter_ready",reason:"Academic 分步生成、答案与媒体校验、独立校审和核对发布；真实连通以配置页测试为准"},
  ];
}
