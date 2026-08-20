# Installs the i binary + PowerShell wrapper from the latest GitHub release.
#   irm https://raw.githubusercontent.com/iwandejong/i/main/install.ps1 | iex
$ErrorActionPreference = "Stop"

$Repo = "iwandejong/i"
$BinDir = if ($env:I_BIN_DIR) { $env:I_BIN_DIR } else { "$HOME\.local\bin" }
$ShareDir = if ($env:I_SHARE_DIR) { $env:I_SHARE_DIR } else { "$HOME\.local\share\i" }

if ($env:PROCESSOR_ARCHITECTURE -ne "AMD64") {
    Write-Error "i: no prebuilt Windows $($env:PROCESSOR_ARCHITECTURE) binary yet — build from source: https://github.com/$Repo#install"
    exit 1
}
$target = "x86_64-pc-windows-msvc"

$url = "https://github.com/$Repo/releases/latest/download/i-$target.tar.gz"
$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ([System.IO.Path]::GetRandomFileName())
New-Item -ItemType Directory -Path $tmp | Out-Null
try {
    Write-Host "Downloading i for $target..."
    $archive = Join-Path $tmp "i.tar.gz"
    Invoke-WebRequest -Uri $url -OutFile $archive
    tar xzf $archive -C $tmp

    New-Item -ItemType Directory -Force -Path $BinDir, $ShareDir | Out-Null
    Copy-Item "$tmp\i-$target\i.exe" "$BinDir\i.exe" -Force
    Copy-Item "$tmp\i-$target\i.ps1" "$ShareDir\i.ps1" -Force
    Write-Host "Installed $BinDir\i.exe"
} finally {
    Remove-Item -Recurse -Force $tmp
}

if (($env:Path -split ";") -notcontains $BinDir) {
    Write-Host "Note: $BinDir isn't on your PATH yet — add it, e.g.:"
    Write-Host "  [Environment]::SetEnvironmentVariable('Path', `"`$env:Path;$BinDir`", 'User')"
}

$scriptPath = "$ShareDir\i.ps1"
if (-not (Test-Path $PROFILE)) {
    New-Item -ItemType File -Force -Path $PROFILE | Out-Null
}
if ((Get-Content $PROFILE -Raw -ErrorAction SilentlyContinue) -like "*$scriptPath*") {
    Write-Host "Shell integration already set up in $PROFILE"
} else {
    Add-Content -Path $PROFILE -Value "`n# i — fuzzy recursive cd (https://github.com/$Repo)`n. `"$scriptPath`""
    Write-Host "Added to $PROFILE — restart PowerShell or run: . `$PROFILE"
}
