$ErrorActionPreference = "Continue"

$testOutput = @(& cargo test 2>&1)
$testExitCode = $LASTEXITCODE
$testOutput | ForEach-Object { Write-Output $_ }

if ($testExitCode -ne 0 -and -not [string]::IsNullOrWhiteSpace($env:GITHUB_STEP_SUMMARY)) {
    $summaryLines = @("## cargo test failed", "", '```text')
    $summaryLines += @($testOutput | ForEach-Object { $_.ToString() })
    $summaryLines += '```'
    $summaryLines | Add-Content -LiteralPath $env:GITHUB_STEP_SUMMARY
}

exit $testExitCode
