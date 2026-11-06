param([switch]$Models)
$ErrorActionPreference = 'Stop'
Push-Location (Split-Path -Parent $PSScriptRoot)
try {
    cargo fmt --all -- --check
    if ($LASTEXITCODE -ne 0) { throw 'Formatting check failed. Run cargo fmt --all.' }
    cargo clippy --locked --all-features --all-targets -- -D warnings
    if ($LASTEXITCODE -ne 0) { throw 'Lint checks failed.' }
    cargo test --locked --all-features
    if ($LASTEXITCODE -ne 0) { throw 'Tests failed.' }
    if ($Models) {
        foreach ($model in @('SaveAfter', 'WritersAfter', 'ImportLifecycle', 'RecordLifecycle', 'CacheWrites', 'RestoreAfter', 'LibrarySaveAfter')) {
            & ./statecharts/run.ps1 -Model $model
        }
    }
    Write-Output 'All requested checks passed.'
} finally { Pop-Location }
