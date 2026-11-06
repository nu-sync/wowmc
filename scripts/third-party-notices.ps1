param(
    [Parameter(Mandatory = $true)][string]$Output,
    [string]$Target = 'x86_64-pc-windows-msvc'
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
Push-Location $root
try {
    $metadataJson = cargo metadata --locked --format-version 1 --no-default-features --filter-platform $Target
    if ($LASTEXITCODE -ne 0) { throw 'Dependency metadata collection failed.' }
    $metadata = $metadataJson | ConvertFrom-Json
    $packages = @{}
    $nodes = @{}
    foreach ($package in $metadata.packages) { $packages[$package.id] = $package }
    foreach ($node in $metadata.resolve.nodes) { $nodes[$node.id] = $node }
    $visited = [Collections.Generic.HashSet[string]]::new()
    $pending = [Collections.Generic.Stack[string]]::new()
    $pending.Push($metadata.resolve.root)
    while ($pending.Count -gt 0) {
        $id = $pending.Pop()
        if (-not $visited.Add($id)) { continue }
        foreach ($dependency in $nodes[$id].deps) {
            if (@($dependency.dep_kinds | Where-Object { $null -eq $_.kind }).Count -gt 0) {
                $pending.Push($dependency.pkg)
            }
        }
    }
    $dependencies = @($visited | Where-Object { $_ -ne $metadata.resolve.root } | ForEach-Object { $packages[$_] } | Sort-Object name, version)
    $text = [Text.StringBuilder]::new()
    [void]$text.AppendLine('Third-party notices for wowmc CLI')
    [void]$text.AppendLine("Target: $Target")
    [void]$text.AppendLine('Includes normal dependencies of the CLI, with default features disabled.')
    [void]$text.AppendLine('License expressions and license texts are supplied by the dependency packages.')
    foreach ($package in $dependencies) {
        $packageRoot = Split-Path -Parent $package.manifest_path
        $licenseFiles = @(Get-ChildItem -LiteralPath $packageRoot -File | Where-Object { $_.Name -match '^(LICEN[CS]E|COPYING|NOTICE)' })
        if ($package.license_file) {
            $declaredFile = if ([IO.Path]::IsPathRooted($package.license_file)) { $package.license_file } else { Join-Path $packageRoot $package.license_file }
            $licenseFiles += Get-Item -LiteralPath $declaredFile
        }
        $licenseFiles = @($licenseFiles | Sort-Object FullName -Unique)
        if ($licenseFiles.Count -eq 0) { throw "No license files found for $($package.name) $($package.version)." }
        [void]$text.AppendLine()
        [void]$text.AppendLine(('=' * 72))
        [void]$text.AppendLine("Package: $($package.name) $($package.version)")
        $license = if ($package.license) { $package.license } else { 'See license text below.' }
        [void]$text.AppendLine("Declared license: $license")
        foreach ($file in $licenseFiles) {
            [void]$text.AppendLine()
            [void]$text.AppendLine("--- $($file.Name) ---")
            [void]$text.AppendLine([IO.File]::ReadAllText($file.FullName).TrimEnd())
        }
    }
    $outputPath = [IO.Path]::GetFullPath($Output)
    $stream = [IO.File]::Open($outputPath, [IO.FileMode]::CreateNew, [IO.FileAccess]::Write, [IO.FileShare]::None)
    try {
        $bytes = [Text.UTF8Encoding]::new($false).GetBytes($text.ToString())
        $stream.Write($bytes, 0, $bytes.Length)
    } finally { $stream.Dispose() }
    Write-Output "Generated notices for $($dependencies.Count) dependencies."
} finally { Pop-Location }
