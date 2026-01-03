param(
  [string]$Output = (Join-Path "sessions/recordings" ("cli-demo-" + (Get-Date -Format "yyyyMMdd-HHmmss") + ".mp4")),
  [int]$Fps = 30,
  [int]$PauseSeconds = 3,
  [int]$ExplainPauseSeconds = 4,
  [int]$SectionPauseSeconds = 5
)

$ErrorActionPreference = "Stop"

$ffmpeg = (Get-Command ffmpeg -ErrorAction SilentlyContinue)?.Source
if (-not $ffmpeg) {
  throw "ffmpeg not found on PATH. Install it and retry."
}

$repoRoot = Split-Path -Parent $PSScriptRoot
$demoScript = Join-Path $PSScriptRoot "run-all-cli.ps1"
if (-not (Test-Path -LiteralPath $demoScript)) {
  throw "Missing demo script: $demoScript"
}

$destDir = Split-Path -Parent $Output
if ($destDir -and -not (Test-Path -LiteralPath $destDir)) {
  New-Item -ItemType Directory -Force -Path $destDir | Out-Null
}

$title = "JWT-Tester-CLI-Demo-" + (Get-Date -Format "yyyyMMdd-HHmmss")

$proc = Start-Process -FilePath "pwsh" -ArgumentList @(
  "-NoProfile",
  "-File", $demoScript,
  "-PauseSeconds", $PauseSeconds,
  "-ExplainPauseSeconds", $ExplainPauseSeconds,
  "-SectionPauseSeconds", $SectionPauseSeconds,
  "-WindowTitle", $title,
  "-Explain"
) -WorkingDirectory $repoRoot -PassThru

$timeout = [TimeSpan]::FromSeconds(10)
$start = Get-Date
while ($proc.MainWindowHandle -eq 0 -and ((Get-Date) - $start) -lt $timeout) {
  Start-Sleep -Milliseconds 200
  $proc.Refresh()
}
if ($proc.MainWindowHandle -eq 0) {
  throw "Demo window did not appear in time."
}

$logPath = $Output + ".ffmpeg.log"
$args = @(
  "-hide_banner",
  "-loglevel", "error",
  "-nostats",
  "-f", "gdigrab",
  "-framerate", $Fps,
  "-i", ("title=" + $title),
  "-vf", "scale=trunc(iw/2)*2:trunc(ih/2)*2",
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

$ffProc = New-Object System.Diagnostics.Process
$ffProc.StartInfo = $psi

if (-not $ffProc.Start()) {
  throw "Failed to start ffmpeg."
}

Start-Sleep -Milliseconds 400
if ($ffProc.HasExited) {
  Start-Sleep -Milliseconds 200
  $ffErr = $ffProc.StandardError.ReadToEnd()
  $ffOut = $ffProc.StandardOutput.ReadToEnd()
  if ($ffErr -or $ffOut) {
    Set-Content -LiteralPath $logPath -Value ($ffErr + $ffOut)
  }
  throw "ffmpeg exited early. See log: $logPath"
}

Write-Host "Recording demo window '$title' -> $Output"

$proc.WaitForExit()

if (-not $ffProc.HasExited) {
  try {
    $ffProc.StandardInput.WriteLine("q")
    $ffProc.StandardInput.Flush()
    $ffProc.WaitForExit(5000) | Out-Null
  } catch {
    $ffProc.Kill()
  }
}

$ffErrFinal = $ffProc.StandardError.ReadToEnd()
$ffOutFinal = $ffProc.StandardOutput.ReadToEnd()
if ($ffErrFinal -or $ffOutFinal) {
  Set-Content -LiteralPath $logPath -Value ($ffErrFinal + $ffOutFinal)
}

if (-not (Test-Path -LiteralPath $Output)) {
  throw "Recording failed; output not found. Check ffmpeg log: $logPath"
}

Write-Host "Recording saved: $Output"
