$vsShell = "C:\Program Files\Microsoft Visual Studio\2022\Community\Common7\Tools\Launch-VsDevShell.ps1"
if (-not (Test-Path $vsShell)) {
  Write-Error "未找到 VS DevShell: $vsShell"
  exit 1
}

$principal = New-Object Security.Principal.WindowsPrincipal([Security.Principal.WindowsIdentity]::GetCurrent())
if (-not $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {
  $args = "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`""
  Start-Process -FilePath "powershell" -Verb RunAs -ArgumentList $args
  exit 0
}

Get-Process -Name "facewinunlock-tauri" -ErrorAction SilentlyContinue | Stop-Process -Force

& $vsShell -Arch amd64 -HostArch amd64 | Out-Null
Set-Location (Join-Path $PSScriptRoot "..")

$vcToolsDir = $env:VCToolsInstallDir
$winSdkDir = $env:WindowsSdkDir
$winSdkVer = $env:WindowsSDKVersion
if (-not $winSdkVer) {
  $winSdkVer = ""
}
$winSdkVer = $winSdkVer.TrimEnd("\")

$includePaths = @()
if ($vcToolsDir) {
  $includePaths += (Join-Path $vcToolsDir "include")
}
if ($winSdkDir -and $winSdkVer) {
  $includePaths += (Join-Path $winSdkDir "Include\$winSdkVer\ucrt")
  $includePaths += (Join-Path $winSdkDir "Include\$winSdkVer\shared")
  $includePaths += (Join-Path $winSdkDir "Include\$winSdkVer\um")
  $includePaths += (Join-Path $winSdkDir "Include\$winSdkVer\winrt")
  $includePaths += (Join-Path $winSdkDir "Include\$winSdkVer\cppwinrt")
}
$includePaths = $includePaths | Where-Object { Test-Path $_ }
$env:OPENCV_CLANG_ARGS = ($includePaths | ForEach-Object { "-I`"$($_)`"" }) -join " "

$opencvDir = "E:\opencv\build"
$opencvBin = "E:\opencv\build\x64\vc16\bin"
$opencvLib = "E:\opencv\build\x64\vc16\lib"
$opencvInclude = "E:\opencv\build\include"
$llvmBin = "C:\Program Files\LLVM\bin"

$env:OPENCV_DIR = $opencvDir
$env:OpenCV_DIR = $opencvDir
$env:OPENCV_INCLUDE_PATHS = $opencvInclude
$env:OPENCV_LINK_PATHS = $opencvLib
$env:OPENCV_LINK_LIBS = "opencv_world4120"
$env:LIBCLANG_PATH = $llvmBin
$env:LLVM_CONFIG_PATH = (Join-Path $llvmBin "llvm-config.exe")
$env:PATH = "$opencvBin;$llvmBin;" + $env:PATH

npm run tauri dev
