param([string]$Output = "dist\wowmc-windows-x64.zip")
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$previousTarget = $env:CARGO_TARGET_DIR
$env:CARGO_TARGET_DIR = Join-Path $root 'target'
Push-Location $root
try {
    $outputPath = if ([IO.Path]::IsPathRooted($Output)) { [IO.Path]::GetFullPath($Output) } else { [IO.Path]::GetFullPath((Join-Path $root $Output)) }
    $checksumPath = "$outputPath.sha256"
    foreach ($path in @($outputPath, $checksumPath)) {
        if (Test-Path -LiteralPath $path) { throw "Output already exists: $path. Choose a new -Output path." }
    }
    $licensePath = Join-Path $root 'LICENSE'
    if (-not (Test-Path -LiteralPath $licensePath -PathType Leaf)) { throw 'The root LICENSE file is required for packaging.' }
    $target = 'x86_64-pc-windows-msvc'
    $hostTriple = (rustc -vV | Select-String '^host:').ToString()
    if ($hostTriple -ne "host: $target") { throw "A Windows x64 MSVC Rust toolchain is required. Found: $hostTriple" }
    cargo build --locked --release --no-default-features --target $target --bin wowmc
    if ($LASTEXITCODE -ne 0) { throw 'Release build failed.' }
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $outputPath) | Out-Null
    $stage = Join-Path $root ('target\package-' + [guid]::NewGuid())
    New-Item -ItemType Directory -Path $stage | Out-Null
    Copy-Item -LiteralPath (Join-Path $root "target\$target\release\wowmc.exe") -Destination $stage
    Copy-Item -LiteralPath (Join-Path $root 'README.md') -Destination $stage
    Copy-Item -LiteralPath $licensePath -Destination $stage
    & (Join-Path $PSScriptRoot 'third-party-notices.ps1') -Output (Join-Path $stage 'THIRD_PARTY_NOTICES.txt') -Target $target
    Compress-Archive -Path (Join-Path $stage '*') -DestinationPath $outputPath
    $hash = (Get-FileHash -LiteralPath $outputPath -Algorithm SHA256).Hash.ToLowerInvariant()
    $checksum = "$hash  $([IO.Path]::GetFileName($outputPath))`n"
    $checksumStream = [IO.File]::Open($checksumPath, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    try {
        $bytes = [Text.UTF8Encoding]::new($false).GetBytes($checksum)
        $checksumStream.Write($bytes, 0, $bytes.Length)
    } finally { $checksumStream.Dispose() }
    Write-Output "Created $outputPath"
    Write-Output "Created $checksumPath"
} finally { Pop-Location; $env:CARGO_TARGET_DIR = $previousTarget }
