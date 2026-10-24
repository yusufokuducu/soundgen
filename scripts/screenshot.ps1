# Capture the SoundGen window to a PNG.
#
#   powershell -ExecutionPolicy Bypass -File scripts/screenshot.ps1
#
# Launches the app with --demo, waits for the window, grabs it with
# PrintWindow (works even if it is partly covered) and writes docs/assets.
# Keep this file ASCII-only: PowerShell 5.1 reads .ps1 as ANSI.

param(
    [string]$Out = "docs/assets/screenshot.png",
    [int]$WaitSeconds = 25,
    [switch]$Reuse
)

$ErrorActionPreference = "Stop"
Add-Type -AssemblyName System.Drawing

Add-Type @"
using System;
using System.Runtime.InteropServices;
public class SgWin32 {
    [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
    [DllImport("user32.dll", CharSet = CharSet.Unicode)] public static extern IntPtr FindWindow(string cls, string title);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern bool PrintWindow(IntPtr h, IntPtr hdc, uint flags);
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
    [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
    [StructLayout(LayoutKind.Sequential)] public struct RECT { public int Left, Top, Right, Bottom; }
}
"@

[SgWin32]::SetProcessDPIAware() | Out-Null

$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$exe = Join-Path $root "target\release\soundgen.exe"
if (-not (Test-Path $exe)) {
    throw "missing $exe - run: cargo build --release -p soundgen"
}

$title = "SoundGen - Procedural SFX"
$proc = $null
if (-not $Reuse) {
    $proc = Start-Process -FilePath $exe -ArgumentList "--demo" -PassThru
}

$hwnd = [IntPtr]::Zero
$deadline = (Get-Date).AddSeconds($WaitSeconds)
while ((Get-Date) -lt $deadline) {
    # [NullString]::Value is a real NULL; a plain $null marshals as "" and
    # FindWindow then looks for a window whose class name is empty.
    $hwnd = [SgWin32]::FindWindow([NullString]::Value, $title)
    if ($hwnd -ne [IntPtr]::Zero -and [SgWin32]::IsWindowVisible($hwnd)) { break }
    Start-Sleep -Milliseconds 300
}
if ($hwnd -eq [IntPtr]::Zero) {
    if ($proc) { Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue }
    throw "window '$title' never appeared"
}

# Restore, raise, and give the UI time to finish its first renders (the
# variation family and the layer thumbnails are scheduled ~0.4 s after load).
[SgWin32]::ShowWindow($hwnd, 9) | Out-Null
[SgWin32]::SetForegroundWindow($hwnd) | Out-Null
Start-Sleep -Milliseconds 2500

$rect = New-Object SgWin32+RECT
[SgWin32]::GetWindowRect($hwnd, [ref]$rect) | Out-Null
$w = $rect.Right - $rect.Left
$h = $rect.Bottom - $rect.Top
if ($w -lt 200 -or $h -lt 200) { throw "bad window rect ${w}x${h}" }

$bmp = New-Object System.Drawing.Bitmap($w, $h)
$gfx = [System.Drawing.Graphics]::FromImage($bmp)
$hdc = $gfx.GetHdc()
# flag 2 = PW_RENDERFULLCONTENT, needed for GPU-composited (OpenGL) windows
$ok = [SgWin32]::PrintWindow($hwnd, $hdc, 2)
$gfx.ReleaseHdc($hdc)
$gfx.Dispose()

if (-not $ok) {
    # Fall back to a plain screen grab of the same rectangle.
    $bmp.Dispose()
    $bmp = New-Object System.Drawing.Bitmap($w, $h)
    $gfx = [System.Drawing.Graphics]::FromImage($bmp)
    $gfx.CopyFromScreen($rect.Left, $rect.Top, 0, 0, (New-Object System.Drawing.Size($w, $h)))
    $gfx.Dispose()
}

$dir = Split-Path -Parent (Join-Path $root $Out)
if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }
$path = Join-Path $root $Out
$bmp.Save($path, [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()

# Reject an all-black grab (happens if the compositor refused PrintWindow).
$probe = New-Object System.Drawing.Bitmap($path)
$pw = $probe.Width
$ph = $probe.Height
$sum = 0
$n = 0
for ($x = 0; $x -lt $pw; $x += 7) {
    for ($y = 0; $y -lt $ph; $y += 7) {
        $c = $probe.GetPixel($x, $y)
        $sum += $c.R + $c.G + $c.B
        $n += 3
    }
}
$probe.Dispose()
$mean = if ($n -gt 0) { $sum / $n } else { 0 }

if ($proc -and -not $Reuse) { Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue }

"wrote $Out (${w}x${h}, mean luma $([int]$mean))"
if ($mean -lt 3) { throw "screenshot looks black" }
