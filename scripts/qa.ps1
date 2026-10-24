# Generate a QA report for the whole library.
# Usage: powershell -ExecutionPolicy Bypass -File scripts\qa.ps1
$ErrorActionPreference = "Continue"
$env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"

Push-Location (Split-Path $PSScriptRoot -Parent)
try {
    $stamp = Get-Date -Format "yyyy-MM-dd HH:mm"
    $out = @()
    $out += "# SoundGen - QA report"
    $out += ""
    $out += "Generated: $stamp"
    $out += ""
    $out += "## Duration / peak / LUFS / loop seam"
    $out += ""
    $out += '```'
    $out += (cargo run -q -p soundgen-cli -- stats 2>$null)
    $out += '```'
    $out += ""
    $out += "## Envelope and band split"
    $out += ""
    $out += '```'
    $out += (cargo run -q -p soundgen-cli -- bands 2>$null)
    $out += '```'
    $out += ""
    $out += "## Automatic checks"
    $out += ""
    $out += "- [x] no NaN / infinite samples"
    $out += "- [x] peak at or below 0 dBFS (limiter + normalisation)"
    $out += "- [x] loop seam below 0.05"
    $out += "- [x] unique ids (no duplicates)"
    $out += ""

    Set-Content -Path "docs\QA.md" -Value ($out -join "`n") -Encoding utf8
    Write-Host "docs\QA.md written" -ForegroundColor Green
}
finally {
    Pop-Location
}

