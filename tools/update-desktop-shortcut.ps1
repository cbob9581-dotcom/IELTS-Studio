param([string]$Version)
$ErrorActionPreference='Stop'
$projectRoot=Split-Path -Parent $PSScriptRoot
if(-not $Version){$Version=(Get-Content -LiteralPath (Join-Path $projectRoot 'app/package.json') -Raw | ConvertFrom-Json).version}
if($Version -notmatch '^\d+\.\d+\.\d+$'){throw 'Invalid desktop version'}
$target=(Resolve-Path -LiteralPath (Join-Path $projectRoot "dist/$Version/ielts-desktop.exe")).Path
if((Get-Item -LiteralPath $target).VersionInfo.ProductVersion -ne $Version){throw 'Executable version does not match delivery version'}
. (Join-Path $PSScriptRoot 'windows-shortcut.ps1')
$desktop=[Environment]::GetFolderPath('Desktop')
$startMenu=Join-Path $env:APPDATA 'Microsoft/Windows/Start Menu/Programs/IELTS Studio.lnk'
$links=@((Join-Path $projectRoot '启动 IELTS Studio.lnk'),(Join-Path $desktop 'IELTS Studio.lnk'),$startMenu)
# Preserve an existing shortcut before updating only this project's entry points.
$backup=Join-Path $projectRoot ('.tools/shortcut-backups/'+[DateTime]::Now.ToString('yyyyMMdd-HHmmss-ffff'))
for($entryIndex=0;$entryIndex -lt $links.Count;$entryIndex++){
 $path=$links[$entryIndex]
 if(Test-Path -LiteralPath $path){
  $oldTarget=[IELTSStudioShortcut]::Target($path)
  if($oldTarget -and -not $oldTarget.StartsWith($projectRoot+'\',[StringComparison]::OrdinalIgnoreCase) -and [IO.Path]::GetFileName($oldTarget) -ne 'ielts-desktop.exe'){throw 'Existing shortcut belongs to a different application; not replaced'}
  New-Item -ItemType Directory -Path $backup -Force | Out-Null
  Copy-Item -LiteralPath $path -Destination (Join-Path $backup ("$entryIndex-"+[IO.Path]::GetFileName($path)))
 }
 [IELTSStudioShortcut]::Save($path,$target,"IELTS Studio $Version")
 if([IELTSStudioShortcut]::Target($path) -ne $target){throw 'Shortcut target verification failed'}
 [pscustomobject]@{Shortcut=$path;Version=$Version;Target=$target}
}
