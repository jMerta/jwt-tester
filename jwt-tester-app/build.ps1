$cliOnly = $false
$ui = $false
$release = $false
$passthrough = @()

foreach ($arg in $args) {
    switch ($arg) {
        '--cli-only' { $cliOnly = $true }
        '--ui' { $ui = $true }
        '--release' { $release = $true }
        default { $passthrough += $arg }
    }
}

if ($cliOnly -and $ui) {
    Write-Error 'Choose either --cli-only or --ui.'
    exit 1
}

$manifest = Join-Path $PSScriptRoot 'Cargo.toml'
$cmd = @('cargo', 'build', '--manifest-path', $manifest)

if ($release) {
    $cmd += '--release'
}

if ($cliOnly) {
    $cmd += '--no-default-features'
    $cmd += '--features'
    $cmd += 'cli-only'
} elseif ($ui) {
    $cmd += '--features'
    $cmd += 'ui'
}

if ($passthrough.Count -gt 0) {
    $cmd += $passthrough
}

Write-Host ('Running: ' + ($cmd -join ' '))
& $cmd
exit $LASTEXITCODE
