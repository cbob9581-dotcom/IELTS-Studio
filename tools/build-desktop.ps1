param([switch]$NoBundle)
$ErrorActionPreference='Stop'
$projectRoot=Split-Path -Parent $PSScriptRoot
Set-Location -LiteralPath (Join-Path $projectRoot 'app')
$env:RUSTUP_TOOLCHAIN='1.97.1-x86_64-pc-windows-gnu'
$env:CARGO_TARGET_DIR=Join-Path $env:TEMP 'ielts-desktop-build'
# MinGW requires an ASCII compiler output path; source remains in D:/雅思.
if($NoBundle){npm run tauri build -- --no-bundle}else{npm run tauri build -- --bundles nsis}
if($LASTEXITCODE -ne 0){throw 'Desktop build failed'}
$package=Get-Content -LiteralPath package.json -Raw | ConvertFrom-Json
$delivery=Join-Path $projectRoot ('dist/'+$package.version)
New-Item -ItemType Directory -Path $delivery -Force | Out-Null
$release=Join-Path $env:CARGO_TARGET_DIR 'release'
Copy-Item -LiteralPath (Join-Path $release 'ielts-desktop.exe') -Destination $delivery -Force
Copy-Item -LiteralPath (Join-Path $release 'WebView2Loader.dll') -Destination $delivery -Force
$assetTarget=Join-Path $delivery 'runtime/speaking'
New-Item -ItemType Directory -Path $assetTarget -Force | Out-Null
foreach($name in @('speaking-room.glb','manifest.json')){Copy-Item -LiteralPath (Join-Path $projectRoot ('rooms/speaking/runtime/'+$name)) -Destination $assetTarget -Force}
foreach($name in @('THIRD_PARTY_NOTICES.txt','ASSET-NOTICES.md')){Copy-Item -LiteralPath (Join-Path $projectRoot ('app/notices/'+$name)) -Destination $delivery -Force}
if(-not $NoBundle){Copy-Item -LiteralPath (Join-Path $release ('bundle/nsis/IELTS Studio_'+$package.version+'_x64-setup.exe')) -Destination $delivery -Force}
Get-ChildItem -LiteralPath $delivery | Select-Object Name,Length
