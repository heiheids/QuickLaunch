# 一键构建：优先使用工程内自带的 Rust 工具链（.rust\），
# 没有的话再退回系统 PATH 里的 cargo。
# 用法： pwsh -File build.ps1            （Release 构建 + 自检）
#        pwsh -File build.ps1 -Test      （先跑单元测试）

param(
    [switch]$Test
)

$ErrorActionPreference = 'Stop'
$root = $PSScriptRoot
Set-Location $root

$localBin = Join-Path $root '.rust\cargo\bin'
if (Test-Path (Join-Path $localBin 'cargo.exe')) {
    $env:RUSTUP_HOME = Join-Path $root '.rust\rustup'
    $env:CARGO_HOME = Join-Path $root '.rust\cargo'
    $env:PATH = $localBin + ';' + $env:PATH
    Write-Host "[build] 使用工程内工具链 .rust\" -ForegroundColor Cyan
} elseif (Get-Command cargo -ErrorAction SilentlyContinue) {
    Write-Host "[build] 使用系统 PATH 中的 cargo" -ForegroundColor Cyan
} else {
    Write-Host "[build] 未找到 cargo：请先安装 Rust（rustup），或用 rustup-init 安装到 .rust\" -ForegroundColor Red
    exit 1
}

$env:CARGO_TERM_COLOR = 'never'

if ($Test) {
    Write-Host "[build] cargo test --release" -ForegroundColor Cyan
    cargo test --release
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

Write-Host "[build] cargo build --release" -ForegroundColor Cyan
cargo build --release
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$exe = Join-Path $root 'target\release\QuickLaunch.exe'
if (-not (Test-Path $exe)) {
    Write-Host "[build] 构建失败：找不到 $exe" -ForegroundColor Red
    exit 1
}

# 自检：直接解析 PE 资源表，逐字节核对内嵌图标与 res\app.ico 是否一致
# （比读文件大小靠谱得多：资源目录写错时大小照样对，图标却是坏的）
Write-Host "[build] 校验内嵌图标" -ForegroundColor Cyan
$verify = cargo run --release --example verify_icon 2>&1 | Out-String
$last = ($verify -split "`r?`n" | Where-Object { $_ -match '结论：' } | Select-Object -Last 1)
if ($last -match '图标资源正确') {
    Write-Host "[build] $last" -ForegroundColor Green
} else {
    Write-Host "[build] $last" -ForegroundColor Yellow
    Write-Host "[build] 如果图标不正确，请检查 res\app.ico 与 build.rs 的资源生成逻辑" -ForegroundColor Yellow
}

$size = [math]::Round((Get-Item $exe).Length / 1KB)
Write-Host ("[build] 产物 {0}  {1} KB" -f $exe, $size) -ForegroundColor Green
