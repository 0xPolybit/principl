[CmdletBinding()]
param(
    [string]$CompilerPath
)

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
if ([string]::IsNullOrWhiteSpace($CompilerPath)) {
    $CompilerPath = Join-Path $repositoryRoot "target\release\princi.exe"
}
$CompilerPath = (Resolve-Path -LiteralPath $CompilerPath).Path
$temporaryRoot = [System.IO.Path]::GetFullPath($env:TEMP).TrimEnd([char[]]@('\', '/'))
$smokeDirectory = Join-Path $temporaryRoot ("Princi Release Smoke " + [guid]::NewGuid().ToString("N"))
$fullSmokeDirectory = [System.IO.Path]::GetFullPath($smokeDirectory)

if (-not $fullSmokeDirectory.StartsWith($temporaryRoot + [System.IO.Path]::DirectorySeparatorChar, [System.StringComparison]::OrdinalIgnoreCase)) {
    throw "Refusing to create smoke-test files outside the temporary directory."
}

New-Item -ItemType Directory -Path $fullSmokeDirectory | Out-Null
Push-Location $fullSmokeDirectory
try {
    @'
fn main() {
    println("Princi release smoke test passed")
}
'@ | Set-Content -LiteralPath ".\hello.prnc" -Encoding Ascii

    Copy-Item -LiteralPath ".\hello.prnc" -Destination ".\hello.princi"

    & $CompilerPath build (Join-Path $fullSmokeDirectory "hello.prnc")
    if ($LASTEXITCODE -ne 0) {
        throw "The .prnc release smoke build failed with exit code $LASTEXITCODE."
    }
    $defaultExecutable = Join-Path $fullSmokeDirectory "hello.exe"
    if (-not (Test-Path -LiteralPath $defaultExecutable -PathType Leaf)) {
        throw "The .prnc release smoke build did not create hello.exe."
    }
    $defaultOutput = (& $defaultExecutable | Out-String).Trim()
    if ($LASTEXITCODE -ne 0 -or $defaultOutput -ne "Princi release smoke test passed") {
        throw "The .prnc executable did not produce the expected output."
    }

    $explicitExecutable = Join-Path $fullSmokeDirectory "app.exe"
    & $CompilerPath build (Join-Path $fullSmokeDirectory "hello.princi") -o $explicitExecutable
    if ($LASTEXITCODE -ne 0) {
        throw "The .princi release smoke build failed with exit code $LASTEXITCODE."
    }
    if (-not (Test-Path -LiteralPath $explicitExecutable -PathType Leaf)) {
        throw "The .princi release smoke build did not create app.exe."
    }
    $explicitOutput = (& $explicitExecutable | Out-String).Trim()
    if ($LASTEXITCODE -ne 0 -or $explicitOutput -ne "Princi release smoke test passed") {
        throw "The .princi executable did not produce the expected output."
    }

    Write-Host "Release smoke test passed for .prnc, .princi, default output, and -o."
}
finally {
    Pop-Location
    if (Test-Path -LiteralPath $fullSmokeDirectory) {
        $resolvedDirectory = [System.IO.Path]::GetFullPath($fullSmokeDirectory)
        if (-not $resolvedDirectory.StartsWith($temporaryRoot + [System.IO.Path]::DirectorySeparatorChar, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "Refusing to remove a smoke-test directory outside the temporary directory."
        }
        Remove-Item -LiteralPath $resolvedDirectory -Recurse -Force
    }
}
