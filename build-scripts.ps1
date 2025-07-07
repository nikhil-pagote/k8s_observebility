#!/usr/bin/env pwsh

<#
.SYNOPSIS
    Build Rust deployment scripts using Docker

.DESCRIPTION
    This script builds the Rust binaries for Kubernetes observability stack management
    using Docker cross-compilation to ensure compatibility across platforms.

.PARAMETER Clean
    Clean build - remove existing binaries before building

.EXAMPLE
    .\build-scripts.ps1
    Builds all Rust binaries

.EXAMPLE
    .\build-scripts.ps1 -Clean
    Cleans existing binaries and rebuilds
#>

param(
    [switch]$Clean
)

# Colors for output
$Green = "Green"
$Yellow = "Yellow"
$Red = "Red"
$Cyan = "Cyan"

function Write-Status {
    param(
        [string]$Message,
        [string]$Color = "White"
    )
    Write-Host $Message -ForegroundColor $Color
}

function Invoke-Command {
    param(
        [string]$Command,
        [string]$Description
    )
    
    Write-Status "🔄 $Description" $Cyan
    Write-Status "📋 Executing: $Command" $Cyan
    
    try {
        $result = Invoke-Expression $Command
        if ($LASTEXITCODE -eq 0) {
            Write-Status "✅ Command completed successfully" $Green
            if ($result) {
                Write-Host $result
            }
        } else {
            Write-Status "❌ Command failed with exit code $LASTEXITCODE" $Red
            exit 1
        }
    }
    catch {
        Write-Status "❌ Command failed: $($_.Exception.Message)" $Red
        exit 1
    }
}

# Check prerequisites
Write-Status "🔍 Checking prerequisites..." $Yellow

$tools = @("docker", "git")
foreach ($tool in $tools) {
    try {
        $null = Get-Command $tool -ErrorAction Stop
        Write-Status "✅ $tool is available" $Green
    }
    catch {
        Write-Status "❌ $tool is required but not installed" $Red
        exit 1
    }
}

# Create bin directory if it doesn't exist
if (!(Test-Path "bin")) {
    Write-Status "📁 Creating bin directory..." $Cyan
    New-Item -ItemType Directory -Path "bin" -Force | Out-Null
}

# Clean existing binaries if requested
if ($Clean) {
    Write-Status "🧹 Cleaning existing binaries..." $Yellow
    if (Test-Path "bin") {
        Get-ChildItem "bin" -Filter "*.exe" | Remove-Item -Force
        Write-Status "✅ Existing binaries removed" $Green
    }
}

# Build Docker image if it doesn't exist
Write-Status "🐳 Checking Docker builder image..." $Yellow
$imageExists = docker images --format "table {{.Repository}}:{{.Tag}}" | Select-String "rust-builder:latest"
if (!$imageExists) {
    Write-Status "🔨 Building Docker builder image..." $Yellow
    Invoke-Command "docker build -f Dockerfile.build -t rust-builder ." "Building Docker builder image"
} else {
    Write-Status "✅ Docker builder image already exists" $Green
}

# Build Rust binaries
Write-Status "🔨 Building Rust deployment scripts using Docker..." $Yellow

$currentDir = (Get-Location).Path.Replace('\', '/')
$dockerCmd = "docker run --rm -v ${currentDir}/src-build:/app -v ${currentDir}/bin:/output rust-builder"

Invoke-Command $dockerCmd "Building Rust binaries in Docker container"

# Verify binaries were created
Write-Status "🔍 Verifying built binaries..." $Yellow
$binaries = @("setup_kind_cluster.exe", "deploy_argocd.exe", "cleanup.exe", "k8s-obs.exe")
$allExist = $true

foreach ($binary in $binaries) {
    $path = "bin/$binary"
    if (Test-Path $path) {
        Write-Status "✅ $binary" $Green
    } else {
        Write-Status "❌ $binary - not found" $Red
        $allExist = $false
    }
}

if ($allExist) {
    Write-Status "✅ All scripts built successfully in bin/ directory" $Green
    Write-Status "🚀 Ready to use with k8s-obs.exe" $Green
} else {
    Write-Status "❌ Some binaries failed to build" $Red
    exit 1
} 