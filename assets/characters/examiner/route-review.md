# Examiner authoring route

The first working examiner is authored inside Blender 5.1.2 with MPFB 2.0.18 source and MakeHuman CC0 assets. Source hashes are recorded in `.tools/character-sources/sources.json` at project root. MPFB writable configuration and downloads remain inside `D:/雅思/.tools/`.

The Blender rig currently has 163 deform bones. The body exposes 22 Microsoft-style viseme controls and 52 ARKit-style face controls, alongside the modelling targets. Relevant deformations are transferred to teeth, tongue, eyes, eyelashes and hair. The Windows test fixture supplies 51 viseme events for a 6.06-second audio clip. Audio output is explicitly 16kHz so event timestamps use the same clock as the WAV file.

This is a first rigged visual iteration, not a claim of final photorealistic likeness to the approved image reference. Face and hair refinement remains subject to review. The selected trouser suit is recolored in shader nodes, and its disconnected tie is masked while keeping source topology intact.

The authoring route works in Blender. No Unreal or Godot installation was found at the checked standard locations. Desktop engine selection and actual runtime performance remain pending; do not claim the 1080p/60 FPS target has been achieved. A Blender-rendered movie does not prove engine import compatibility or performance.

No DeepSeek, Qwen ASR or Qwen TTS call has been made. The test voice is Windows Microsoft Zira, used only to validate the rig. A cloud TTS adapter must preserve the shared audio playback clock and provide viseme data or use a separate audio-to-viseme service.

Sources:
- https://static.makehumancommunity.org/about/license.html
- https://static.makehumancommunity.org/assets/assetpacks/index.html
- https://static.makehumancommunity.org/assets/assetpacks/suits01.html
- https://static.makehumancommunity.org/mpfb/docs/exporting/export_copy.html
- https://github.com/makehumancommunity/mpfb2

Before engine export, bake the character's identity/modelling targets while retaining facial animation controls; remove non-render helper geometry and validate bone weights and animation again. Keep the native editable authoring rig in the Blender source.
