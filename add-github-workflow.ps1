<#
.SYNOPSIS
    Adds GitHub Actions CI workflow to an existing Rust project
.DESCRIPTION
    Adds a comprehensive Rust CI/CD workflow to a project's .github/workflows directory
.PARAMETER ProjectPath
    Path to the existing Rust project root directory
#>

param (
    [string]$ProjectPath = (Get-Location).Path
)

# Ensure the project path exists
if (-not (Test-Path $ProjectPath)) {
    Write-Host "Error: Project path does not exist." -ForegroundColor Red
    exit 1
}

# Check if it's a Rust project (has Cargo.toml)
$cargoTomlPath = Join-Path $ProjectPath "Cargo.toml"
if (-not (Test-Path $cargoTomlPath)) {
    Write-Host "Error: Not a Rust project. Cargo.toml not found." -ForegroundColor Red
    exit 1
}

# Create .github/workflows directory if it doesn't exist
$workflowsPath = Join-Path $ProjectPath ".github\workflows"
New-Item -Path $workflowsPath -ItemType Directory -Force | Out-Null

# CI Workflow content
$ciWorkflowContent = @"
name: Rust CI/CD

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  lint:
    name: Lint
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          components: rustfmt, clippy
          override: true
      - name: Check formatting
        run: cargo fmt -- --check
      - name: Clippy
        run: cargo clippy -- -D warnings

  test:
    name: Test
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: `${{ matrix.os }}
    steps:
      - uses: actions/checkout@v3
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      - name: Install JACK (Linux)
        if: runner.os == 'Linux'
        run: |
          sudo apt-get update
          sudo apt-get install -y libjack-jackd2-dev
      - name: Install JACK (macOS)
        if: runner.os == 'macOS'
        run: brew install jack
      - name: Build
        run: cargo build --verbose
      - name: Run tests
        run: cargo test --verbose

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true
      - name: Run cargo-tarpaulin
        uses: actions-rs/tarpaulin@v0.1
        with:
          version: '0.22.0'
          args: '--verbose --all-features --workspace --timeout 120 --out Xml'
      - name: Upload to codecov.io
        uses: codecov/codecov-action@v3
"@

# Write the workflow file
$workflowFilePath = Join-Path $workflowsPath "rust-ci.yml"
Set-Content -Path $workflowFilePath -Value $ciWorkflowContent

# Confirm successful addition
Write-Host "GitHub Actions workflow added successfully:" -ForegroundColor Green
Write-Host "  Path: $workflowFilePath" -ForegroundColor Cyan

# Optional: Suggest next steps
Write-Host "`nNext steps:" -ForegroundColor Yellow
Write-Host "1. Review the workflow file" -ForegroundColor White
Write-Host "2. Commit and push to your repository" -ForegroundColor White
Write-Host "3. Enable GitHub Actions in your repository settings" -ForegroundColor White