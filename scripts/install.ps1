#Requires -Version 5.1
<#
.SYNOPSIS
  Install mcsm from GitHub Releases (CntierTeam/MCSManager-Client).

.EXAMPLE
  # Latest stable
  irm https://raw.githubusercontent.com/CntierTeam/MCSManager-Client/main/scripts/install.ps1 | iex

.EXAMPLE
  # Continuous build from main
  & ([scriptblock]::Create((irm https://raw.githubusercontent.com/CntierTeam/MCSManager-Client/main/scripts/install.ps1))) -Continuous

.EXAMPLE
  .\scripts\install.ps1 -Version v0.1.0
  .\scripts\install.ps1 -InstallDir 'C:\Tools\mcsm'
#>
[CmdletBinding()]
param(
    [ValidateSet('stable', 'continuous')]
    [string]$Channel = $(if ($env:MCSM_CHANNEL) { $env:MCSM_CHANNEL } else { 'stable' }),

    [switch]$Continuous,
    [switch]$Nightly,

    [string]$Version = $(if ($env:MCSM_VERSION) { $env:MCSM_VERSION } else { '' }),

    [string]$InstallDir = $(
        if ($env:MCSM_INSTALL_DIR) { $env:MCSM_INSTALL_DIR }
        else { Join-Path $env:LOCALAPPDATA 'Programs\mcsm' }
    ),

    [string]$Repo = $(if ($env:MCSM_REPO) { $env:MCSM_REPO } else { 'CntierTeam/MCSManager-Client' }),

    [switch]$NoVerify,
    [switch]$NoPath
)

$ErrorActionPreference = 'Stop'

if ($Continuous -or $Nightly) { $Channel = 'continuous' }

function Get-CpuArch {
    $a = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString().ToLowerInvariant()
    switch -Regex ($a) {
        'x64|amd64' { return 'amd64' }
        'arm64' { return 'arm64' }
        default {
            # Fallback for older PowerShell
            if ($env:PROCESSOR_ARCHITECTURE -match 'ARM64') { return 'arm64' }
            return 'amd64'
        }
    }
}

function Resolve-Tag {
    param([string]$ApiBase)
    if ($Version) { return $Version }
    if ($Channel -eq 'continuous') { return 'continuous' }

    try {
        $latest = Invoke-RestMethod -Uri "$ApiBase/latest" -Headers @{ 'User-Agent' = 'mcsm-install' }
        if ($latest.tag_name) { return [string]$latest.tag_name }
    } catch {
        # fall through
    }

    $releases = Invoke-RestMethod -Uri "$ApiBase`?per_page=20" -Headers @{ 'User-Agent' = 'mcsm-install' }
    $stable = $releases | Where-Object { -not $_.prerelease -and $_.tag_name -like 'v*' } | Select-Object -First 1
    if (-not $stable) {
        throw "No stable release found; try -Continuous or -Version vX.Y.Z"
    }
    return [string]$stable.tag_name
}

function Add-UserPath {
    param([string]$Dir)
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    if (-not $userPath) { $userPath = '' }
    $parts = $userPath -split ';' | Where-Object { $_ -and $_.Trim() -ne '' }
    if ($parts -contains $Dir) { return }
    $newPath = ($parts + $Dir) -join ';'
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
    $env:Path = "$Dir;$env:Path"
    Write-Host "Added to user PATH: $Dir"
}

$cpu = Get-CpuArch
if ($cpu -ne 'amd64') {
    Write-Warning "Current Release builds ship windows-amd64 only; detected $cpu — download may fail."
}

$artifact = "mcsm-windows-$cpu"
$archive = "$artifact.zip"
$api = "https://api.github.com/repos/$Repo/releases"
$tag = Resolve-Tag -ApiBase $api
$assetUrl = "https://github.com/$Repo/releases/download/$tag/$archive"
$sumUrl = "$assetUrl.sha256"

Write-Host "repo=$Repo tag=$tag artifact=$artifact"
Write-Host "download: $assetUrl"

$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("mcsm-install-" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmp | Out-Null
try {
    $zipPath = Join-Path $tmp $archive
    Invoke-WebRequest -Uri $assetUrl -OutFile $zipPath -UseBasicParsing

    if (-not $NoVerify) {
        try {
            $sumPath = Join-Path $tmp "$archive.sha256"
            Invoke-WebRequest -Uri $sumUrl -OutFile $sumPath -UseBasicParsing
            $expected = ((Get-Content -Raw $sumPath) -split '\s+')[0].Trim().ToLowerInvariant()
            $actual = (Get-FileHash -Algorithm SHA256 -Path $zipPath).Hash.ToLowerInvariant()
            if ($expected -ne $actual) {
                throw "sha256 mismatch: expected=$expected actual=$actual"
            }
            Write-Host "$archive : OK"
        } catch {
            if ($_.Exception.Message -match 'sha256 mismatch') { throw }
            Write-Warning "checksum file missing or unverifiable; skip verify"
        }
    }

    $extract = Join-Path $tmp 'extract'
    Expand-Archive -Path $zipPath -DestinationPath $extract -Force
    $bin = Get-ChildItem -Path $extract -Recurse -File |
        Where-Object { $_.Name -eq 'mcsm.exe' -or $_.Name -eq 'mcsm' } |
        Select-Object -First 1
    if (-not $bin) {
        throw "mcsm.exe not found inside archive"
    }

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    $dest = Join-Path $InstallDir 'mcsm.exe'
    Copy-Item -Force $bin.FullName $dest
    Write-Host "installed: $dest"

    if (-not $NoPath) {
        Add-UserPath -Dir $InstallDir
    }

    try {
        & $dest --help | Select-Object -First 3
    } catch {
        Write-Host "(binary installed; open a new terminal if 'mcsm' is not found yet)"
    }

    Write-Host "OK"
    Write-Host "Run: mcsm --help"
} finally {
    Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
}
