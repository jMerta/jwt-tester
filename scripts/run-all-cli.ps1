param(
  [string]$JwtTester = "jwt-tester",
  [string]$DataRoot = (Join-Path "sessions/recordings" ("cli-test-data-" + (Get-Date -Format "yyyyMMdd-HHmmss"))),
  [int]$PauseSeconds = 2,
  [int]$ExplainPauseSeconds = 3,
  [int]$SectionPauseSeconds = 4,
  [string]$WindowTitle,
  [switch]$Explain
)

$ErrorActionPreference = "Stop"

$ColorHeading = "Cyan"
$ColorCommand = "Yellow"
$ColorExplain = "Green"
$ColorNote = "DarkGray"

function Pause {
  param([int]$Seconds)
  if ($Seconds -gt 0) {
    Start-Sleep -Seconds $Seconds
  }
}

function Write-Section {
  param([string]$Title)
  Write-Host ""
  Write-Host ("==== " + $Title + " ====") -ForegroundColor $ColorHeading
  Pause $SectionPauseSeconds
}

function Invoke-JwtTester {
  param(
    [string]$Label,
    [string[]]$CommandArgs,
    [switch]$Capture
  )

  $stepLabel = ("[{0}] {1}" -f $script:StepIndex, $Label)
  Write-Host ""
  Write-Host $stepLabel -ForegroundColor $ColorHeading
  Write-Host ("$JwtTester " + ($CommandArgs -join " ")) -ForegroundColor $ColorCommand
  Pause $PauseSeconds

  if ($Capture) {
    $output = & $JwtTester @CommandArgs
    if ($LASTEXITCODE -ne 0) { throw "Command failed: $Label" }
    return $output
  }

  & $JwtTester @CommandArgs
  if ($LASTEXITCODE -ne 0) { throw "Command failed: $Label" }
  $script:StepIndex++
}

function Explain {
  param([string]$Text)
  if ($Explain) {
    Write-Host ""
    Write-Host $Text -ForegroundColor $ColorExplain
    Pause $ExplainPauseSeconds
  }
}

if (-not (Test-Path -LiteralPath $DataRoot)) {
  New-Item -ItemType Directory -Force -Path $DataRoot | Out-Null
}

if ($WindowTitle) {
  $host.UI.RawUI.WindowTitle = $WindowTitle
}

$script:StepIndex = 1

Write-Host ""
Write-Host "JWT Tester CLI walkthrough" -ForegroundColor $ColorHeading
Write-Host "We run core commands and a vault workflow." -ForegroundColor $ColorNote
Write-Host "All data is isolated under sessions/recordings." -ForegroundColor $ColorNote
Pause $SectionPauseSeconds

$dataDir = Join-Path $DataRoot "vault"
New-Item -ItemType Directory -Force -Path $dataDir | Out-Null

$env:JWT_TESTER_KEYCHAIN_SERVICE = "jwt-tester-cli-recording"
$env:JWT_TESTER_TEST_SECRET = "recording-secret"
$env:JWT_TESTER_EXPORT_PASSPHRASE = "recording-export-passphrase"

$common = @("--data-dir", $dataDir)
$project = "demo-project"
$keyName = "demo-key"
$genKeyName = "generated-key"
$tokenName = "demo-token"

Explain "Tip: pass --json before the subcommand to get machine-readable output."

Write-Section "Basics"
$versionArgs = @("--version")
Invoke-JwtTester -Label "Version" -CommandArgs $versionArgs

Explain "Generating shell completions to show integration with your shell."
$completionFile = Join-Path $DataRoot "jwt-tester-completion.ps1"
$completionArgs = $common + @("completion", "powershell")
$completion = Invoke-JwtTester -Label "Completion (PowerShell)" -CommandArgs $completionArgs -Capture
Set-Content -LiteralPath $completionFile -Value $completion
Write-Host "Saved completion to $completionFile"
Write-Host "Completion preview:"
($completion -split "`n" | Select-Object -First 3) | ForEach-Object { Write-Host $_ }
Write-Host "Tip: dot-source the file to enable completion in the current shell." -ForegroundColor $ColorNote

Write-Section "Token operations"
Explain "Creating a demo token with HS256, then decoding, inspecting, splitting, and verifying it."
$encodeArgs = $common + @(
  "encode",
  "--alg", "hs256",
  "--secret", "env:JWT_TESTER_TEST_SECRET",
  "--sub", "demo-user",
  "--exp", "1h"
)
$token = Invoke-JwtTester -Label "Encode token (HS256)" -CommandArgs $encodeArgs -Capture
$token = ($token -join "`n").Trim()

$tokenFile = Join-Path $DataRoot "token.jwt"
Set-Content -LiteralPath $tokenFile -Value $token
Write-Host "Token saved to $tokenFile"

$decodeArgs = $common + @("decode", $token)
Invoke-JwtTester -Label "Decode token" -CommandArgs $decodeArgs
$inspectArgs = $common + @("inspect", $token)
Invoke-JwtTester -Label "Inspect token" -CommandArgs $inspectArgs
$splitArgs = $common + @("split", $token)
Invoke-JwtTester -Label "Split token" -CommandArgs $splitArgs
$verifyArgs = $common + @("verify", "--alg", "hs256", "--secret", "env:JWT_TESTER_TEST_SECRET", $token)
Invoke-JwtTester -Label "Verify token" -CommandArgs $verifyArgs

Write-Section "Vault workflow"
Explain "Now we use the vault: create a project, add keys/tokens, export/import, then clean up."
$projectAddArgs = $common + @("vault", "project", "add", $project, "--description", "Demo project")
Invoke-JwtTester -Label "Vault project add" -CommandArgs $projectAddArgs
$projectListArgs = $common + @("vault", "project", "list", "--details")
Invoke-JwtTester -Label "Vault project list" -CommandArgs $projectListArgs

$keyAddArgs = $common + @(
  "vault", "key", "add",
  "--project", $project,
  "--name", $keyName,
  "--kind", "hmac",
  "--secret", "env:JWT_TESTER_TEST_SECRET",
  "--description", "Demo key"
)
Invoke-JwtTester -Label "Vault key add" -CommandArgs $keyAddArgs

$keyGenArgs = $common + @(
  "vault", "key", "generate",
  "--project", $project,
  "--name", $genKeyName,
  "--kind", "hmac"
)
Invoke-JwtTester -Label "Vault key generate" -CommandArgs $keyGenArgs

$keyListArgs = $common + @("vault", "key", "list", "--project", $project, "--details")
Invoke-JwtTester -Label "Vault key list" -CommandArgs $keyListArgs
$setDefaultArgs = $common + @("vault", "project", "set-default-key", "--project", $project, "--key-name", $keyName)
Invoke-JwtTester -Label "Vault project set-default-key" -CommandArgs $setDefaultArgs

$tokenAddArgs = $common + @("vault", "token", "add", "--project", $project, "--name", $tokenName, "--token", $token)
Invoke-JwtTester -Label "Vault token add" -CommandArgs $tokenAddArgs
$tokenListArgs = $common + @("vault", "token", "list", "--project", $project, "--details")
Invoke-JwtTester -Label "Vault token list" -CommandArgs $tokenListArgs

$exportFile = Join-Path $DataRoot "vault-export.json"
$exportArgs = $common + @("vault", "export", "--passphrase", "env:JWT_TESTER_EXPORT_PASSPHRASE", "--out", $exportFile)
Invoke-JwtTester -Label "Vault export" -CommandArgs $exportArgs
$importArgs = $common + @("vault", "import", "--bundle", ("@" + $exportFile), "--passphrase", "env:JWT_TESTER_EXPORT_PASSPHRASE", "--replace")
Invoke-JwtTester -Label "Vault import (replace)" -CommandArgs $importArgs

$tokenDeleteArgs = $common + @("vault", "token", "delete", "--project", $project, "--name", $tokenName)
Invoke-JwtTester -Label "Vault token delete" -CommandArgs $tokenDeleteArgs
$keyDeleteDemoArgs = $common + @("vault", "key", "delete", "--project", $project, "--name", $keyName)
Invoke-JwtTester -Label "Vault key delete (demo)" -CommandArgs $keyDeleteDemoArgs
$keyDeleteGenArgs = $common + @("vault", "key", "delete", "--project", $project, "--name", $genKeyName)
Invoke-JwtTester -Label "Vault key delete (generated)" -CommandArgs $keyDeleteGenArgs
$projectDeleteArgs = $common + @("vault", "project", "delete", "--name", $project)
Invoke-JwtTester -Label "Vault project delete" -CommandArgs $projectDeleteArgs

Write-Section "Local UI"
Explain "Finally, we start the local UI briefly to show it launches."
$uiOutLog = Join-Path $DataRoot "ui.out.log"
$uiErrLog = Join-Path $DataRoot "ui.err.log"
$uiArgs = $common + @("ui", "--port", "0")
Write-Host ""
Write-Host ("[{0}] UI start (brief)" -f $script:StepIndex) -ForegroundColor $ColorHeading
Write-Host ("$JwtTester " + ($uiArgs -join " ")) -ForegroundColor $ColorCommand
Pause $PauseSeconds
$script:StepIndex++
$jwtCommand = Get-Command $JwtTester -ErrorAction Stop
if ($jwtCommand.CommandType -eq "ExternalScript") {
  $uiFile = "pwsh"
  $uiArgList = @("-NoProfile", "-File", $jwtCommand.Source) + $uiArgs
} else {
  $uiFile = $JwtTester
  $uiArgList = $uiArgs
}
$uiProc = Start-Process -FilePath $uiFile -ArgumentList $uiArgList -PassThru -NoNewWindow -RedirectStandardOutput $uiOutLog -RedirectStandardError $uiErrLog
Start-Sleep -Seconds 3
if (-not $uiProc.HasExited) {
  Stop-Process -Id $uiProc.Id -Force
}
Write-Host "UI stdout (last 10 lines):"
if (Test-Path -LiteralPath $uiOutLog) {
  Get-Content -LiteralPath $uiOutLog -Tail 10
}
Write-Host "UI stderr (last 10 lines):"
if (Test-Path -LiteralPath $uiErrLog) {
  Get-Content -LiteralPath $uiErrLog -Tail 10
}

Write-Host ""
Write-Host "All CLI commands executed."
