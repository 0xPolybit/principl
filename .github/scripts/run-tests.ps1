$ErrorActionPreference = "Continue"

$testOutput = @(& cargo test 2>&1)
$testExitCode = $LASTEXITCODE

if ($testExitCode -ne 0) {
    $failureLines = @($testOutput | Where-Object {
        $_.ToString() -match '^(test result: FAILED|test .* \.\.\. FAILED|error(?:\[[^]]+\])?:|failures:|Caused by:|thread .+ panicked at)'
    } | Select-Object -First 30 | ForEach-Object { $_.ToString() })
    if ($failureLines.Count -eq 0) {
        $failureLines = @($testOutput | Select-Object -Last 20 | ForEach-Object { $_.ToString() })
    }

    if (-not [string]::IsNullOrWhiteSpace($env:GITHUB_STEP_SUMMARY)) {
        $summaryLines = @("## cargo test failed", "", '```text') + $failureLines + @('```', '')
        $summaryLines | Add-Content -LiteralPath $env:GITHUB_STEP_SUMMARY
    }

    $annotation = ($failureLines -join ' | ').Replace('%', '%25').Replace(':', '%3A').Replace(',', '%2C').Replace("`r", '%0D').Replace("`n", '%0A')
    if ($annotation.Length -gt 3000) {
        $annotation = $annotation.Substring(0, 3000)
    }
    Write-Output "::error title=Princi compiler tests::$annotation"
    exit $testExitCode
}

Write-Output "All compiler tests passed."
