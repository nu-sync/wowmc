param(
    [Parameter(Mandatory=$true)]
    [ValidateSet('SaveBefore','SaveAfter','WritersBefore','WritersAfter','ImportLifecycle','RecordLifecycle','CacheWrites','RestoreBefore','RestoreAfter','LibrarySaveBefore','LibrarySaveAfter')]
    [string]$Model,
    [string]$Out = (Join-Path ([IO.Path]::GetTempPath()) ('wowmc-model-' + [guid]::NewGuid())),
    [switch]$ExpectViolation
)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$Out = [IO.Path]::GetFullPath($Out)
if (Test-Path -LiteralPath $Out) { throw 'Choose a new output folder.' }
$previousJar = $env:TLA2TOOLS
$jar = if ($env:TLA2TOOLS) { $env:TLA2TOOLS } else { Join-Path $root 'target/tools/tla2tools-1.7.4.jar' }
$hash = '936a262061c914694dfd669a543be24573c45d5aa0ff20a8b96b23d01e050e88'
if (-not (Test-Path -LiteralPath $jar)) {
    if ($previousJar) { throw 'TLA2TOOLS does not point to an existing jar.' }
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $jar) | Out-Null
    $download = "$jar.$([guid]::NewGuid()).download"
    Invoke-WebRequest -Uri 'https://github.com/tlaplus/tlaplus/releases/download/v1.7.4/tla2tools.jar' -OutFile $download
    if ((Get-FileHash -LiteralPath $download -Algorithm SHA256).Hash -ne $hash) { throw 'TLC download checksum mismatch.' }
    Copy-Item -LiteralPath $download -Destination $jar
    Remove-Item -LiteralPath $download
}
if ((Get-FileHash -LiteralPath $jar -Algorithm SHA256).Hash -ne $hash) { throw 'Expected the pinned TLC 1.7.4 jar; checksum mismatch.' }
$env:TLA2TOOLS = (Resolve-Path -LiteralPath $jar).Path
$views = @{ SaveBefore='save-view.mjs'; SaveAfter='after-view.mjs'; WritersBefore='writers-view.mjs'; WritersAfter='writers-after-view.mjs'; ImportLifecycle='import-view.mjs'; RecordLifecycle='record-view.mjs'; CacheWrites='cache-view.mjs'; RestoreBefore='restore-view.mjs'; RestoreAfter='restore-view.mjs'; LibrarySaveBefore='library-save-view.mjs'; LibrarySaveAfter='library-save-view.mjs' }
$spec = if ($Model.StartsWith('Restore')) { 'RestoreLifecycle' } elseif ($Model.StartsWith('LibrarySave')) { 'LibrarySave' } else { $Model }
$stage = Join-Path ([IO.Path]::GetTempPath()) ('wowmc-statechart-' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $stage | Out-Null
try {
    foreach ($name in @("$spec.tla", "$Model.cfg", $views[$Model])) {
        Copy-Item -LiteralPath (Join-Path $PSScriptRoot $name) -Destination $stage
    }
    & node --disable-warning=ExperimentalWarning (Join-Path $root 'tools/statechart/worker.ts') $stage (Join-Path $stage "$spec.tla") (Join-Path $stage "$Model.cfg") (Join-Path $stage $views[$Model])
    $result = $LASTEXITCODE
    if ($result -notin @(0, 1)) { throw "Model runner failed ($result). Scratch files: $stage" }
    New-Item -ItemType Directory -Path $Out -Force | Out-Null
    Copy-Item -Path (Join-Path $stage 'done/*') -Destination $Out
    Get-Content -LiteralPath (Join-Path $Out 'report.md')
    Write-Output "Artifacts: $Out"
    $expected = if ($ExpectViolation) { 1 } else { 0 }
    if ($result -ne $expected) { throw "Expected model exit $expected but got $result. See $Out" }
} finally { $env:TLA2TOOLS = $previousJar }
