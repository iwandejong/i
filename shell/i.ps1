# i — fuzzy recursive cd, scoped to your current directory.
#
# Dot-source this from your PowerShell profile ($PROFILE):
#   . /path/to/i.ps1
#
# Then just run:
#   i test         # cd to the best fuzzy match for "test" under $PWD
#   i ../foo       # scoped to ../ first, then fuzzy-searching "foo"
#   i te<TAB>      # cycle through the top matches (PSReadLine tab completion)
#   i -2           # cd up 2 directories (../..), same idea as pushd -N
#
# The underlying `i` binary only prints matching paths on stdout; it never
# touches your shell's directory on its own, which is why this wrapper
# exists.

function i {
    param(
        [Parameter(ValueFromRemainingArguments = $true)]
        [string[]]$Args
    )

    if (-not $Args -or $Args.Count -eq 0) {
        Set-Location $HOME
        return
    }

    switch ($Args[0]) {
        { $_ -in '--help', '-h', '--version', '-V', '--config' } {
            & i.exe @Args
            return
        }
    }

    if ($Args.Count -eq 1 -and $Args[0] -match '^-(\d+)$') {
        $n = [int]$Matches[1]
        $up = ('..' + [IO.Path]::DirectorySeparatorChar) * $n
        Set-Location $up
        return
    }

    if ($Args.Count -eq 1 -and (Test-Path -PathType Container $Args[0])) {
        Set-Location $Args[0]
        return
    }

    $dest = & i.exe @Args
    if ($LASTEXITCODE -eq 0 -and $dest) {
        Set-Location $dest
    }
}

# `i te<TAB>` — offers the top fuzzy matches via PSReadLine tab completion.
# No highlighting/menu-select like the zsh version, just a plain candidate
# list cycled with Tab.
Register-ArgumentCompleter -CommandName i -Native -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)
    & i.exe --complete -- $wordToComplete 2>$null | ForEach-Object {
        [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_)
    }
}
