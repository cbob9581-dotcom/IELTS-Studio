export const providerIds = [
  "deepseek",
  "qwenAsr",
  "qwenTts",
  "mineru",
  "evaluation",
] as const;
export type ProviderId = (typeof providerIds)[number];
export type ProviderConfig = {
  enabled: boolean;
  baseUrl: string;
  model: string;
  region: string;
  voice: string;
  language: string;
  timeoutSeconds: number;
  temperature: number;
  maxTokens: number;
};
export type AppConfig = {
  schemaVersion: 1;
  providers: Record<ProviderId, ProviderConfig>;
  audio: {
    inputDeviceId: string;
    outputDeviceId: string;
    masterVolume: number;
    ttsVolume: number;
    sampleRate: number;
    format: string;
  };
  noise: {
    profile: string;
    volume: number;
    handwriting: boolean;
    keyboard: boolean;
    waterPump: boolean;
    speakingEnabled: boolean;
  };
  blender: {
    executablePath: string;
    sourceBlendPath: string;
    runtimeAssetPath: string;
    renderScale: number;
  };
};
export const providerLabels: Record<ProviderId, string> = {
  deepseek: "DeepSeek · 对话与出题",
  qwenAsr: "Qwen ASR · 语音识别",
  qwenTts: "Qwen TTS · 自然语音",
  mineru: "MinerU · 资料解析",
  evaluation: "口语音频评测",
};
export function defaultConfig(): AppConfig {
  return {
    schemaVersion: 1,
    providers: Object.fromEntries(
      providerIds.map((id) => [
        id,
        {
          enabled: false,
          baseUrl: "",
          model: "",
          region: "",
          voice: "",
          language: "en",
          timeoutSeconds: 60,
          temperature: 0.7,
          maxTokens: 4096,
        },
      ]),
    ) as Record<ProviderId, ProviderConfig>,
    audio: {
      inputDeviceId: "",
      outputDeviceId: "",
      masterVolume: 0.75,
      ttsVolume: 0.85,
      sampleRate: 24000,
      format: "wav",
    },
    noise: {
      profile: "clean",
      volume: 0.15,
      handwriting: true,
      keyboard: true,
      waterPump: true,
      speakingEnabled: false,
    },
    blender: {
      executablePath:
        "C:/Program Files/Blender Foundation/Blender 5.1/blender.exe",
      sourceBlendPath: "",
      runtimeAssetPath: "",
      renderScale: 1,
    },
  };
}
