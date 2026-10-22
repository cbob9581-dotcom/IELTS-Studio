# IELTS Studio

IELTS Studio 是一个以全屏 3D 考场为核心的 Academic 雅思桌面练习项目。听力、阅读和写作在房间里的电脑界面中作答；口语在 3D 考官、题卡和纸笔场景中进行。主页负责选择考试或训练、管理题库和配置服务。

当前桌面版本为 0.6.1。项目包含 Windows Tauri 桌面端、3D 口语房间与考官资产、云端出题和评测工作流、资料导入，以及听力脚本合成和音频上传。

## 开发环境

- Windows 10/11
- Node.js 与 npm
- Rust stable toolchain
- Microsoft Edge WebView2 Runtime
- Blender 5.1 用于编辑和导出 3D 场景

在 `app` 目录执行 `npm ci` 安装前端依赖；`npm run dev` 启动前端开发服务。桌面开发使用 `npm run tauri dev`，发行构建使用仓库中的 `tools/build-desktop.ps1`。

前端检查可运行 `npm test -- --run --maxWorkers=2` 和 `npm run build`。Rust 核心测试可从 `app` 目录运行 `cargo test --manifest-path src-tauri/Cargo.toml --no-default-features`。

## 云服务配置

在应用的“设置”页面填写 DeepSeek、DashScope/Qwen 和 MinerU 的服务地址、模型与密钥。API 密钥通过 Windows 凭证管理器保存。不要把 `.env`、密钥、个人录音、上传的真题或本地学习记录提交到仓库。

本地资料、学习记录和生成的试卷由应用按用户选择的位置保存。仓库不包含用户个人资料或已生成的安装包。

## 3D 资源

`blender/speaking_room.blend` 是口语场景源文件，`rooms/speaking/runtime/speaking-room.glb` 是桌面端运行时模型。GLB 和 Blender 文件体积较大，更新场景时请同步检查运行时导出与 `rooms/speaking/runtime/manifest.json`。

出题先完成文字、图表和独立校审，听力 TTS 需要单独启动。失败的单部分文字可单独重新整理；整本题册通过真题资料库按 Test 整理，旧单卷导入的解析缓存可以复用。

0.6.0 提供原生持久任务队列、请求预算、分批资料恢复、证据定位、逐题修复和场景资源缓存。具体云服务参数与恢复步骤见[配置说明](docs/系统优化与配置-0.6.0.md)。口语分数仍为未经人工语料校准的 AI 参考分。

0.6.1 修正阅读按篇显示题目、合格生成卷自动入库和真题使用流程，桌面构建后同步更新启动快捷方式。使用方法与本次验证范围见[使用流程修复说明](docs/使用流程修复-0.6.1.md)。
