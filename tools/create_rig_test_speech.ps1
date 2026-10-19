$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Speech
$testCode = @'
using System;
using System.Collections.Generic;
using System.Globalization;
using System.IO;
using System.Speech.Synthesis;
using System.Speech.AudioFormat;
public class IeltsRigSpeech {
    public static string Create(string wavPath) {
        var events = new List<string>();
        using (var synth = new SpeechSynthesizer()) {
            foreach (var installed in synth.GetInstalledVoices()) {
                var info = installed.VoiceInfo;
                if (installed.Enabled && info.Culture.TwoLetterISOLanguageName == "en" && info.Gender == VoiceGender.Female) {
                    synth.SelectVoice(info.Name); break;
                }
            }
            synth.Rate = 0;
            synth.VisemeReached += (sender, e) => {
                events.Add("{\"time_ms\":" + e.AudioPosition.TotalMilliseconds.ToString("F1",CultureInfo.InvariantCulture) +
                    ",\"duration_ms\":" + e.Duration.TotalMilliseconds.ToString("F1",CultureInfo.InvariantCulture) +
                    ",\"viseme\":" + e.Viseme + "}");
            };
            synth.SetOutputToWaveFile(wavPath, new SpeechAudioFormatInfo(16000, AudioBitsPerSample.Sixteen, AudioChannel.Mono));
            synth.Speak("Good morning. My name is Emma. Can you please tell me your full name?");
            synth.SetOutputToNull();
            return "{\"voice\":\"" + synth.Voice.Name + "\",\"purpose\":\"offline rig validation only; Qwen TTS is not connected\",\"events\":[" + String.Join(",", events) + "]}";
        }
    }
}
'@
Add-Type -TypeDefinition $testCode -ReferencedAssemblies System.Speech
$audioDir = Join-Path (Split-Path -Parent $PSScriptRoot) 'assets\audio'
[System.IO.Directory]::CreateDirectory($audioDir) | Out-Null
$json = [IeltsRigSpeech]::Create((Join-Path $audioDir 'rig-test.wav'))
[System.IO.File]::WriteAllText((Join-Path $audioDir 'rig-test-visemes.json'),$json,[System.Text.Encoding]::UTF8)
Write-Output 'Rig test audio and viseme timestamps saved.'
