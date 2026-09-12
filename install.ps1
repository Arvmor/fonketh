<#
.SYNOPSIS
    Fonketh installer for Windows.

.DESCRIPTION
    Downloads the latest release from GitHub, unpacks it into the install
    directory (default %LOCALAPPDATA%\Programs\Fonketh), adds a `fonketh`
    launcher to your PATH, creates Start Menu / Desktop shortcuts and helps
    you set up a miner key.

    One-liner (PowerShell 5.1 or newer):

        irm https://raw.githubusercontent.com/Arvmor/fonketh/master/install.ps1 | iex

    When piped like that, parameters cannot be passed, so the same settings
    are also read from environment variables: FONKETH_HOME, FONKETH_VERSION,
    FONKETH_YES=1, FONKETH_NO_MODIFY_PATH=1.

.PARAMETER Version
    Release tag to install (e.g. v0.2.0). Defaults to the latest release.
.PARAMETER InstallDir
    Where to install the game.
.PARAMETER Archive
    Install from an already downloaded release .zip instead of GitHub.
.PARAMETER Yes
    Accept all defaults, never prompt.
.PARAMETER NoModifyPath
    Do not change the user PATH.
.PARAMETER Uninstall
    Remove the game (keeps the private key unless -Purge).
#>
[CmdletBinding()]
param(
    [string]$Version = $env:FONKETH_VERSION,
    [string]$InstallDir = $env:FONKETH_HOME,
    [string]$Archive,
    [switch]$Yes,
    [switch]$NoModifyPath,
    [switch]$Uninstall,
    [switch]$Purge
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$Repo = if ($env:FONKETH_REPO) { $env:FONKETH_REPO } else { 'Arvmor/fonketh' }
if (-not $InstallDir) { $InstallDir = Join-Path $env:LOCALAPPDATA 'Programs\Fonketh' }
if ($env:FONKETH_YES -eq '1') { $Yes = $true }
if ($env:FONKETH_NO_MODIFY_PATH -eq '1') { $NoModifyPath = $true }
$Interactive = (-not $Yes) -and (-not [Console]::IsInputRedirected)

# ---------------------------------------------------------------- output ----

function Write-Step([string]$Text) { Write-Host ''; Write-Host '==> ' -ForegroundColor Cyan -NoNewline; Write-Host $Text -ForegroundColor White }
function Write-Ok([string]$Text)   { Write-Host '  ✓ ' -ForegroundColor Green -NoNewline; Write-Host $Text }
function Write-Note([string]$Text) { Write-Host "  $Text" -ForegroundColor DarkGray }
function Write-Warn([string]$Text) { Write-Host '  ! ' -ForegroundColor Yellow -NoNewline; Write-Host $Text }
function Fail([string]$Text) { Write-Host ''; Write-Host 'error: ' -ForegroundColor Red -NoNewline; Write-Host $Text; exit 1 }

function Show-Banner {
    Write-Host @'
   ___          _        _   _
  | __|__ _ __ | |__ ___| |_| |__
  | _/ _ \ ' \ | / // -_)  _| ' \
  |_|\___/_||_||_\_\\___|\__|_||_|
'@ -ForegroundColor Cyan
    Write-Host "  P2P mining pool, gamified. https://github.com/$Repo"
}

# --------------------------------------------------------------- prompts ----

function Ask([string]$Question, [string]$Default) {
    if (-not $Interactive) { return $Default }
    $answer = Read-Host "  $Question [$Default]"
    if ([string]::IsNullOrWhiteSpace($answer)) { $Default } else { $answer.Trim() }
}

function Confirm-Step([string]$Question, [bool]$Default = $true) {
    if (-not $Interactive) { return $Default }
    $hint = if ($Default) { 'Y/n' } else { 'y/N' }
    $answer = Read-Host "  $Question [$hint]"
    if ([string]::IsNullOrWhiteSpace($answer)) { return $Default }
    return $answer.Trim().ToLower().StartsWith('y')
}

function Choose([string]$Question, [string]$Default, [string[]]$Options) {
    foreach ($o in $Options) { Write-Host "    $o" }
    Ask $Question $Default
}

# --------------------------------------------------------------- helpers ----

function Get-Target {
    $arch = $env:PROCESSOR_ARCHITECTURE
    if ($env:PROCESSOR_ARCHITEW6432) { $arch = $env:PROCESSOR_ARCHITEW6432 }
    switch ($arch) {
        'AMD64' { return 'x86_64-pc-windows-msvc' }
        'ARM64' {
            Write-Warn 'No native ARM64 build yet, installing the x64 build (runs under emulation)'
            return 'x86_64-pc-windows-msvc'
        }
        default { Fail "unsupported architecture: $arch" }
    }
}

# Follows github.com/<repo>/releases/latest -> /releases/tag/<tag> without the API
function Resolve-LatestVersion {
    $url = "https://github.com/$Repo/releases/latest"
    try {
        $request = [System.Net.HttpWebRequest]::Create($url)
        $request.Method = 'HEAD'
        $request.AllowAutoRedirect = $false
        $request.UserAgent = 'fonketh-installer'
        $response = $request.GetResponse()
        $location = $response.Headers['Location']
        $response.Close()
    } catch {
        Fail "could not reach GitHub to look up the latest release: $($_.Exception.Message)"
    }
    if (-not $location -or $location -notmatch '/tag/(v[^/]+)$') {
        Fail "could not determine the latest release (got '$location')"
    }
    return $Matches[1]
}

function Get-RemoteFile([string]$Url, [string]$Dest) {
    try {
        Invoke-WebRequest -Uri $Url -OutFile $Dest -UseBasicParsing -Headers @{ 'User-Agent' = 'fonketh-installer' }
        return $true
    } catch {
        return $false
    }
}

function New-RandomHexKey {
    $bytes = New-Object byte[] 32
    $rng = [System.Security.Cryptography.RandomNumberGenerator]::Create()
    $rng.GetBytes($bytes)
    $rng.Dispose()
    return -join ($bytes | ForEach-Object { $_.ToString('x2') })
}

function ConvertFrom-SecureToPlain([securestring]$Secure) {
    $ptr = [Runtime.InteropServices.Marshal]::SecureStringToBSTR($Secure)
    try { [Runtime.InteropServices.Marshal]::PtrToStringBSTR($ptr) }
    finally { [Runtime.InteropServices.Marshal]::ZeroFreeBSTR($ptr) }
}

function New-Shortcut([string]$Path, [string]$Target, [string]$WorkingDir) {
    $shell = New-Object -ComObject WScript.Shell
    $shortcut = $shell.CreateShortcut($Path)
    $shortcut.TargetPath = $Target
    $shortcut.WorkingDirectory = $WorkingDir
    $shortcut.IconLocation = "$Target,0"
    $shortcut.Description = 'Fonketh, the gamified P2P mining pool'
    $shortcut.Save()
}

# ---------------------------------------------------------------- install ---

function Get-ReleaseArchive([string]$Target, [string]$Tmp) {
    $name = "fonketh-$Version-$Target"
    if ($Archive) {
        if (-not (Test-Path $Archive)) { Fail "archive not found: $Archive" }
        Write-Ok "Using local archive $Archive"
        return (Resolve-Path $Archive).Path
    }

    $base = "https://github.com/$Repo/releases/download/$Version"
    $zip = Join-Path $Tmp "$name.zip"
    Write-Host "  Downloading $name.zip"
    if (-not (Get-RemoteFile "$base/$name.zip" $zip)) {
        Fail @"
release $Version has no build for $Target.
  Releases older than the installer shipped bare binaries; pick a newer one with
  -Version, or build from source:
    git clone https://github.com/$Repo; cd fonketh; cargo run -p game_app -F interface --release
"@
    }
    $size = [math]::Round((Get-Item $zip).Length / 1MB, 1)
    Write-Ok "Downloaded ${size} MB"

    $sums = Join-Path $Tmp 'SHA256SUMS.txt'
    if (Get-RemoteFile "$base/SHA256SUMS.txt" $sums) {
        $line = Get-Content $sums | Where-Object { $_ -match "\s$([regex]::Escape("$name.zip"))$" } | Select-Object -First 1
        if (-not $line) {
            Write-Warn 'Archive missing from SHA256SUMS.txt, skipping checksum verification'
        } else {
            $expected = ($line -split '\s+')[0].ToLower()
            $actual = (Get-FileHash -Algorithm SHA256 $zip).Hash.ToLower()
            if ($expected -ne $actual) { Fail "checksum mismatch for $name.zip`n  expected $expected`n  got      $actual" }
            Write-Ok 'Checksum verified'
        }
    } else {
        Write-Warn 'Release has no SHA256SUMS.txt, skipping checksum verification'
    }
    return $zip
}

function Install-Files([string]$Zip, [string]$Tmp) {
    $unpack = Join-Path $Tmp 'unpack'
    Expand-Archive -Path $Zip -DestinationPath $unpack -Force
    $src = Get-ChildItem $unpack -Directory | Select-Object -First 1
    if (-not $src -or -not (Test-Path (Join-Path $src.FullName 'bin\fonketh.exe'))) { Fail 'unexpected archive layout (no bin\fonketh.exe)' }
    if (-not (Test-Path (Join-Path $src.FullName 'assets'))) { Fail 'unexpected archive layout (no assets\)' }

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    # Replace the game files, keep everything else (private key, ...)
    foreach ($d in 'bin', 'assets') {
        $dest = Join-Path $InstallDir $d
        if (Test-Path $dest) { Remove-Item $dest -Recurse -Force }
        Copy-Item (Join-Path $src.FullName $d) $dest -Recurse
    }
    foreach ($f in 'README.md', 'VERSION') {
        $p = Join-Path $src.FullName $f
        if (Test-Path $p) { Copy-Item $p $InstallDir -Force }
    }
    if (-not (Test-Path (Join-Path $InstallDir 'VERSION'))) { Set-Content (Join-Path $InstallDir 'VERSION') $Version }
    # Drop the "downloaded from the internet" mark so SmartScreen is less noisy
    Get-ChildItem (Join-Path $InstallDir 'bin') | Unblock-File -ErrorAction SilentlyContinue
    Write-Ok "Game files installed to $InstallDir"
}

function Install-Launcher {
    # The game keeps its private key and sprites next to itself, so run it from there
    $launcher = Join-Path $InstallDir 'fonketh.cmd'
    @"
@echo off
rem Fonketh launcher, generated by install.ps1
cd /d "%~dp0"
"%~dp0bin\fonketh.exe" %*
"@ | Set-Content -Path $launcher -Encoding ASCII
    Write-Ok "Launcher installed to $launcher"
}

function Update-UserPath {
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $parts = @($userPath -split ';' | Where-Object { $_ })
    if ($parts -contains $InstallDir) { Write-Ok "$InstallDir is already on your PATH"; return }
    if ($NoModifyPath) { Write-Warn "$InstallDir is not on your PATH, add it to run 'fonketh' from any terminal"; return }
    if (-not (Confirm-Step "Add $InstallDir to your PATH?" $true)) { Write-Warn 'Skipped'; return }
    [Environment]::SetEnvironmentVariable('Path', (($parts + $InstallDir) -join ';'), 'User')
    $env:Path = "$env:Path;$InstallDir"
    Write-Ok 'PATH updated (new terminals pick it up)'
}

function Install-Shortcuts {
    $exe = Join-Path $InstallDir 'bin\fonketh.exe'
    $programs = [Environment]::GetFolderPath('Programs')
    try {
        New-Shortcut (Join-Path $programs 'Fonketh.lnk') $exe $InstallDir
        Write-Ok 'Start Menu shortcut created'
    } catch { Write-Warn "Could not create Start Menu shortcut: $($_.Exception.Message)" }
    if (Confirm-Step 'Create a Desktop shortcut?' $true) {
        try {
            New-Shortcut (Join-Path ([Environment]::GetFolderPath('Desktop')) 'Fonketh.lnk') $exe $InstallDir
            Write-Ok 'Desktop shortcut created'
        } catch { Write-Warn "Could not create Desktop shortcut: $($_.Exception.Message)" }
    }
}

function Set-MinerKey {
    $keyFile = Join-Path $InstallDir 'private.key'
    if (Test-Path $keyFile) { Write-Ok "Keeping the existing miner key at $keyFile"; return }

    Write-Host '  Your rewards go to the wallet derived from this key.'
    $choice = Choose 'Miner key' '1' @(
        '1) Generate a new key now (recommended)',
        '2) Import an existing private key',
        '3) Skip, the game creates one on first launch'
    )
    switch ($choice) {
        '2' {
            if (-not $Interactive) { Fail '-Yes cannot import a key' }
            $secure = Read-Host '  Paste the 32-byte hex private key (input hidden)' -AsSecureString
            $key = (ConvertFrom-SecureToPlain $secure).Trim() -replace '^0[xX]', ''
            if ($key -notmatch '^[0-9a-fA-F]{64}$') { Fail 'that is not a 64-character hex private key' }
        }
        '3' { Write-Note "The game will write $keyFile on first launch"; return }
        default { $key = New-RandomHexKey }
    }
    [IO.File]::WriteAllText($keyFile, "0x$key")
    # Only the current user may read the key
    try {
        $acl = Get-Acl $keyFile
        $acl.SetAccessRuleProtection($true, $false)
        $acl.Access | ForEach-Object { $acl.RemoveAccessRule($_) | Out-Null }
        $rule = New-Object Security.AccessControl.FileSystemAccessRule(
            [Security.Principal.WindowsIdentity]::GetCurrent().Name, 'FullControl', 'Allow')
        $acl.AddAccessRule($rule)
        Set-Acl $keyFile $acl
    } catch { Write-Warn "Could not restrict permissions on the key file: $($_.Exception.Message)" }
    Write-Ok "Miner key saved to $keyFile"
    Write-Warn 'Back this file up. Anyone with it controls your rewards; nobody can recover it for you.'
}

function Show-Summary {
    Write-Step "Fonketh $Version is installed"
    Write-Host "  Game files : $InstallDir"
    Write-Host "  Launcher   : $(Join-Path $InstallDir 'fonketh.cmd')"
    Write-Host "  Miner key  : $(Join-Path $InstallDir 'private.key')"
    Write-Host ''
    Write-Host '  Run ' -NoNewline; Write-Host 'fonketh' -ForegroundColor White -NoNewline
    Write-Host ' in a new terminal or use the Start Menu shortcut.'
    Write-Note 'Windows may show a SmartScreen warning on first launch (the binary is not signed): More info -> Run anyway.'
    Write-Note 'Allow the firewall prompt so peers can reach you on UDP port 7331.'
    Write-Host "  Uninstall with: irm https://raw.githubusercontent.com/$Repo/master/install.ps1 | iex   (with `$env:FONKETH_UNINSTALL=1)"
    if (Confirm-Step 'Launch Fonketh now?' $false) {
        Start-Process -FilePath (Join-Path $InstallDir 'bin\fonketh.exe') -WorkingDirectory $InstallDir
    }
}

function Invoke-Uninstall {
    Show-Banner
    Write-Step 'Uninstalling Fonketh'
    if (-not (Test-Path $InstallDir)) { Fail "nothing to uninstall at $InstallDir" }
    if (-not (Confirm-Step "Remove the game files in $InstallDir and the shortcuts?" $true)) { exit 0 }
    foreach ($item in 'bin', 'assets', 'README.md', 'VERSION', 'fonketh.cmd') {
        $p = Join-Path $InstallDir $item
        if (Test-Path $p) { Remove-Item $p -Recurse -Force }
    }
    foreach ($lnk in (Join-Path ([Environment]::GetFolderPath('Programs')) 'Fonketh.lnk'),
                     (Join-Path ([Environment]::GetFolderPath('Desktop')) 'Fonketh.lnk')) {
        if (Test-Path $lnk) { Remove-Item $lnk -Force }
    }
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $parts = @($userPath -split ';' | Where-Object { $_ -and $_ -ne $InstallDir })
    [Environment]::SetEnvironmentVariable('Path', ($parts -join ';'), 'User')
    Write-Ok 'Game files removed'

    $keyFile = Join-Path $InstallDir 'private.key'
    if (Test-Path $keyFile) {
        if ($Purge -or (Confirm-Step "Also delete the miner key $keyFile? This cannot be undone" $false)) {
            Remove-Item $keyFile -Force
            Write-Ok 'Miner key deleted'
        } else {
            Write-Note "Kept $keyFile"
        }
    }
    if ((Get-ChildItem $InstallDir -Force | Measure-Object).Count -eq 0) { Remove-Item $InstallDir -Force }
    Write-Host ''
}

# ------------------------------------------------------------------ main ----

if ($env:FONKETH_UNINSTALL -eq '1') { $Uninstall = $true }
if ($Version -and -not $Version.StartsWith('v')) { $Version = "v$Version" }

if ($Uninstall) { Invoke-Uninstall; exit 0 }

$Tmp = Join-Path ([IO.Path]::GetTempPath()) ("fonketh-install-" + [IO.Path]::GetRandomFileName())
New-Item -ItemType Directory -Path $Tmp | Out-Null
try {
    Show-Banner

    Write-Step 'Step 1/5  Checking your system'
    if ($PSVersionTable.PSVersion.Major -lt 5) { Fail 'PowerShell 5.1 or newer is required' }
    $Target = Get-Target
    Write-Ok "Detected Windows ($Target)"

    Write-Step 'Step 2/5  Picking a release'
    if (-not $Archive) {
        if (-not $Version) { $Version = Resolve-LatestVersion }
        Write-Ok "Release $Version"
        $installed = Join-Path $InstallDir 'VERSION'
        if ((Test-Path $installed) -and ((Get-Content $installed -Raw).Trim() -eq $Version)) {
            Write-Ok "Already installed at $InstallDir"
            if (-not (Confirm-Step 'Reinstall anyway?' $false)) { exit 0 }
        }
    } else {
        if (-not $Version) {
            # fonketh-<tag>-<target>.zip, tags may contain dashes so strip the known target
            $leaf = [IO.Path]::GetFileNameWithoutExtension($Archive) -replace '^fonketh-', '' -replace "-$([regex]::Escape($Target))$", ''
            $Version = if ($leaf.StartsWith('v')) { $leaf } else { 'local' }
        }
        Write-Ok "Release $Version (local archive)"
    }

    Write-Step 'Step 3/5  Choosing where to install'
    $InstallDir = [IO.Path]::GetFullPath((Ask 'Install directory' $InstallDir))
    if ($Interactive) {
        if (-not (Confirm-Step "Install Fonketh $Version to $InstallDir?" $true)) { Write-Host '  Aborted.'; exit 0 }
    } else {
        Write-Ok "Installing to $InstallDir"
    }

    Write-Step 'Step 4/5  Downloading and installing'
    $zip = Get-ReleaseArchive $Target $Tmp
    Install-Files $zip $Tmp
    Install-Launcher
    Update-UserPath
    Install-Shortcuts

    Write-Step 'Step 5/5  Miner key'
    Set-MinerKey

    Show-Summary
} finally {
    Remove-Item $Tmp -Recurse -Force -ErrorAction SilentlyContinue
}
