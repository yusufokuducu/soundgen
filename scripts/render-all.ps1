# Render the whole SoundGen library to WAV with variations and print QA stats.
# Usage: powershell -ExecutionPolicy Bypass -File scripts\render-all.ps1 [-Variations 4] [-Style 0.35] [-Bits 16]
param(
    [int]$Variations = 4,
    [double]$Style = 0.35,
    [int]$Bits = 16,
    [string]$Out = "out"
)

$ErrorActionPreference = "Stop"
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"

Push-Location (Split-Path $PSScriptRoot -Parent)
try {
    Write-Host "== stats ==" -ForegroundColor Cyan
    cargo run -q -p soundgen-cli -- stats

    Write-Host "== render ==" -ForegroundColor Cyan
    cargo run -q --release -p soundgen-cli -- render --out $Out --variations $Variations --style $Style --bits $Bits

    $files = Get-ChildItem -Path $Out -Recurse -Filter *.wav
    $mb = [math]::Round((($files | Measure-Object -Property Length -Sum).Sum / 1MB), 2)
    Write-Host "$($files.Count) WAV files, $mb MB in $Out" -ForegroundColor Green
}
finally {
    Pop-Location
}
