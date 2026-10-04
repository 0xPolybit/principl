[CmdletBinding()]
param(
    [string]$Version
)

$ErrorActionPreference = "Stop"
$repositoryRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$manifestPath = Join-Path $repositoryRoot "Cargo.toml"
$manifest = Get-Content -LiteralPath $manifestPath -Raw
$versionMatch = [regex]::Match($manifest, '(?m)^version\s*=\s*"([^"]+)"')
if (-not $versionMatch.Success) {
    throw "Could not read the Princi package version from Cargo.toml."
}

$packageVersion = $versionMatch.Groups[1].Value
if ([string]::IsNullOrWhiteSpace($Version)) {
    $Version = $packageVersion
}
$Version = $Version.TrimStart("v")
if ($Version -ne $packageVersion) {
    throw "Release version '$Version' does not match Cargo.toml version '$packageVersion'."
}

$releaseExecutable = Join-Path $repositoryRoot "target\release\princi.exe"
if (-not (Test-Path -LiteralPath $releaseExecutable -PathType Leaf)) {
    throw "Missing $releaseExecutable. Run 'cargo build --release' first."
}

$distributionDirectory = Join-Path $repositoryRoot "dist"
$portableDirectory = Join-Path $distributionDirectory "princi-$Version-windows-x64"
$zipPath = Join-Path $distributionDirectory "princi-$Version-windows-x64.zip"
$installerScript = Join-Path $PSScriptRoot "princi.iss"

New-Item -ItemType Directory -Force -Path $distributionDirectory | Out-Null
New-Item -ItemType Directory -Force -Path $portableDirectory | Out-Null
Copy-Item -LiteralPath $releaseExecutable -Destination $portableDirectory -Force
Copy-Item -LiteralPath (Join-Path $repositoryRoot "README.md") -Destination $portableDirectory -Force
Copy-Item -LiteralPath (Join-Path $repositoryRoot "LICENSE") -Destination $portableDirectory -Force
Compress-Archive -Path (Join-Path $portableDirectory "*") -DestinationPath $zipPath -Force

$compiler = Get-Command iscc.exe -ErrorAction SilentlyContinue
if ($null -ne $compiler) {
    $compilerPath = $compiler.Source
}
else {
    $compilerPaths = @(
        (Join-Path ${env:ProgramFiles(x86)} "Inno Setup 7\ISCC.exe"),
        (Join-Path ${env:ProgramFiles(x86)} "Inno Setup 6\ISCC.exe"),
        (Join-Path $env:ProgramFiles "Inno Setup 7\ISCC.exe"),
        (Join-Path $env:ProgramFiles "Inno Setup 6\ISCC.exe")
    )
    $compilerPath = $compilerPaths | Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } | Select-Object -First 1
}

if ([string]::IsNullOrWhiteSpace($compilerPath)) {
    throw "Inno Setup's ISCC.exe was not found. Install Inno Setup 6 or 7, then rerun this script."
}

$versionArgument = "-dAppVersion=$Version"
& $compilerPath $versionArgument $installerScript
if ($LASTEXITCODE -ne 0) {
    throw "Inno Setup failed with exit code $LASTEXITCODE."
}

$installerPath = Join-Path $distributionDirectory "Princi-Setup-$Version-x64.exe"
if (-not (Test-Path -LiteralPath $installerPath -PathType Leaf)) {
    throw "Inno Setup did not produce $installerPath."
}

$checksumPath = Join-Path $distributionDirectory "SHA256SUMS.txt"
$checksumLines = @($zipPath, $installerPath) | ForEach-Object {
    $hash = Get-FileHash -LiteralPath $_ -Algorithm SHA256
    "{0} *{1}" -f $hash.Hash.ToLowerInvariant(), (Split-Path -Leaf $_)
}
$checksumLines | Set-Content -LiteralPath $checksumPath -Encoding Ascii

Write-Host "Created release assets:"
Write-Host "  $zipPath"
Write-Host "  $installerPath"
Write-Host "  $checksumPath"
