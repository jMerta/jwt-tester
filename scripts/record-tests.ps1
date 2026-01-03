param(
  [Parameter(Mandatory = $true, Position = 0)]
  [string]$TestCommand,

  [string]$Output = (Join-Path "sessions/recordings" ("cli-test-" + (Get-Date -Format "yyyyMMdd-HHmmss") + ".mp4")),
  [string]$WindowTitle = "JWT-Tester-Recording",
  [int]$Fps = 30,
  [string]$FfmpegLog
)

$ErrorActionPreference = "Stop"

$ffmpeg = (Get-Command ffmpeg -ErrorAction SilentlyContinue)?.Source
if (-not $ffmpeg) {
  throw "ffmpeg not found on PATH. Install it and retry."
}

$destDir = Split-Path -Parent $Output
if ($destDir -and -not (Test-Path -LiteralPath $destDir)) {
  New-Item -ItemType Directory -Force -Path $destDir | Out-Null
}

$logPath = $FfmpegLog
if (-not $logPath) {
  $logPath = Join-Path (Split-Path -Parent $Output) ([IO.Path]::GetFileNameWithoutExtension($Output) + ".ffmpeg.log")
}

$originalTitle = $host.UI.RawUI.WindowTitle
$host.UI.RawUI.WindowTitle = $WindowTitle

Add-Type @"
using System;
using System.Runtime.InteropServices;
public static class Win32 {
  [DllImport("user32.dll")]
  public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")]
  public static extern bool GetWindowRect(IntPtr hWnd, out RECT rect);
  public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
}
"@

$rect = New-Object Win32+RECT
$hwnd = [Win32]::GetForegroundWindow()
[void][Win32]::GetWindowRect($hwnd, [ref]$rect)
$width = [Math]::Max(0, $rect.Right - $rect.Left)
$height = [Math]::Max(0, $rect.Bottom - $rect.Top)
$width = $width - ($width % 2)
$height = $height - ($height % 2)

$useDesktop = ($width -lt 100 -or $height -lt 100)
if ($useDesktop) {
  Write-Host "Warning: could not resolve active window size. Falling back to full desktop capture."
}

$args = @(
  "-hide_banner",
  "-loglevel", "error",
  "-nostats",
  "-f", "gdigrab",
  "-framerate", $Fps
)

if (-not $useDesktop) {
  $args += @(
    "-offset_x", $rect.Left,
    "-offset_y", $rect.Top,
    "-video_size", ("{0}x{1}" -f $width, $height)
  )
}

$args += @(
  "-i", "desktop",
  "-pix_fmt", "yuv420p",
  "-movflags", "+faststart",
  "-y",
  $Output
)

$psi = New-Object System.Diagnostics.ProcessStartInfo
$psi.FileName = $ffmpeg
$psi.Arguments = ($args -join " ")
$psi.RedirectStandardInput = $true
$psi.RedirectStandardOutput = $true
$psi.RedirectStandardError = $true
$psi.UseShellExecute = $false
$psi.CreateNoWindow = $true

$proc = New-Object System.Diagnostics.Process
$proc.StartInfo = $psi

$exitCode = 0
try {
  if (-not $proc.Start()) {
    throw "Failed to start ffmpeg."
  }

  Start-Sleep -Milliseconds 400
  if ($proc.HasExited) {
    Start-Sleep -Milliseconds 200
    $ffmpegErr = $proc.StandardError.ReadToEnd()
    $ffmpegOut = $proc.StandardOutput.ReadToEnd()
    if ($ffmpegErr -or $ffmpegOut) {
      Set-Content -LiteralPath $logPath -Value ($ffmpegErr + $ffmpegOut)
    }
    throw "ffmpeg exited early. See log: $logPath"
  }

  Write-Host "Recording terminal window '$WindowTitle' -> $Output"
  Write-Host "Running: $TestCommand"

  Invoke-Expression $TestCommand
  if ($LASTEXITCODE) {
    $exitCode = $LASTEXITCODE
  }
} catch {
  $exitCode = 1
  throw
} finally {
  if ($proc -and -not $proc.HasExited) {
    try {
      $proc.StandardInput.WriteLine("q")
      $proc.StandardInput.Flush()
      $proc.WaitForExit(5000) | Out-Null
    } catch {
      $proc.Kill()
    }
  }

  if ($proc) {
    $ffmpegErr = $proc.StandardError.ReadToEnd()
    $ffmpegOut = $proc.StandardOutput.ReadToEnd()
    if ($ffmpegErr -or $ffmpegOut) {
      Set-Content -LiteralPath $logPath -Value ($ffmpegErr + $ffmpegOut)
    }
  }

  $host.UI.RawUI.WindowTitle = $originalTitle
}

if (-not (Test-Path -LiteralPath $Output)) {
  throw "Recording failed; output not found. Check ffmpeg log: $logPath"
}

if ($exitCode -ne 0) {
  exit $exitCode
}
