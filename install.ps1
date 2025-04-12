# QitOps Agent Installation Script for Windows
# This script builds the QitOps Agent and installs it to a location in the PATH

# Build the release version
Write-Host "Building QitOps Agent..." -ForegroundColor Cyan
cargo build --release

# Check if the build was successful
if (-not $?) {
    Write-Host "Build failed. Please check the errors above." -ForegroundColor Red
    exit 1
}

# Create the installation directory if it doesn't exist
$installDir = "$env:USERPROFILE\.qitops\bin"
if (-not (Test-Path $installDir)) {
    Write-Host "Creating installation directory: $installDir" -ForegroundColor Cyan
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

# Copy the binary to the installation directory
Write-Host "Installing QitOps Agent to $installDir" -ForegroundColor Cyan
Copy-Item -Path "target\release\qitops.exe" -Destination "$installDir\qitops.exe" -Force

# Check if the installation directory is in the PATH
$path = [Environment]::GetEnvironmentVariable("Path", "User")
if (-not $path.Contains($installDir)) {
    Write-Host "Adding installation directory to PATH" -ForegroundColor Cyan
    [Environment]::SetEnvironmentVariable("Path", "$path;$installDir", "User")
    $env:Path = "$env:Path;$installDir"
    Write-Host "Added $installDir to PATH" -ForegroundColor Green
}

Write-Host "QitOps Agent has been installed successfully!" -ForegroundColor Green
Write-Host "You can now use the 'qitops' command from any terminal." -ForegroundColor Green
Write-Host "Try 'qitops --help' to get started." -ForegroundColor Cyan
