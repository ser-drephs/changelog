<#
.SYNOPSIS
Build a changelog based on conventional git commits.

.EXAMPLE
PS> Build-Changelog

Builds a CHANGELOG.md for the current repository based on conventional git commits.

.EXAMPLE
PS> Build-Changelog -IncludeAll

Builds a CHANGELOG.md for the current repository based on git commits. Includes all commits.

.EXAMPLE
PS> Build-Changelog -Version "1.2.0"

Builds a CHANGELOG.md for the current repository based on git commits. Sets the headline to "v1.2.0"

.EXAMPLE
PS> Build-Changelog "C:\sources\my-repo" -Version "1.2.0"

Builds a CHANGELOG.md for `my-repo` located at `C:\sources\my-repo` based on git commits. Sets the headline to "v1.2.0"

#>
function Build-Changelog {
    [CmdletBinding()]
    param (
        # Set a different repository
        [Parameter(Position = 0)][string]$Repository,
        # Include all commits not only conventional
        [Parameter()][switch]$IncludeAll,
        # The version to be build
        [Parameter()][ValidatePattern("^[0-9]{1,2}\.[0-9]{1,2}\.[0-9]{1,2}$")][String]$Version
    )
    if (-not $Repository) { $Repository = Get-Location }
    Assert-IsRepository -Repository $Repository

    $relevant = @("feat", "fix", "revert")
    try {
        Push-Location $Repository
        Write-Debug "get commits of repository: $Repository"
        $remoteUrl = & git remote get-url --push origin
        Write-Debug "remote url for this repository: $remoteUrl"
        $commitUrl = ("{0}/commit" -f $remoteUrl)
    }
    finally {
        Pop-Location
    }

    $changelogFile = Join-Path $Repository "Changelog.md"
    Write-Debug "try creating changelog file at $changelogFile"
    # Todo: Read last Tag and set the start point to that date
    if (Test-Path $changelogFile) {
        $changelog = Get-Item $changelogFile
        $range = "--since=`"{0}`"" -f $changelog.LastWriteTime.ToString("yyyy-MM-dd HH:mm:ss")
        $commits = Get-Commits $range
    }
    else {
        $commits = Get-Commits
    }
    Write-Debug ("found commits: {0}" -f $commits.Count)
    $commit = $commits[-1]
    Write-Debug "commit used for first commit: $commit"
    Write-Debug "commit of type: $($commit.GetType())"
    if (-not $commit) {
        Write-debug "commit was empty, using -2"
        $commit = $commits[-2]
    }

    $commitHash = ($commit -split ";")[1]
    try {
        $parentOfFirstCommit = & git log --pretty=%P -n 1 "`"$commitHash`""
    }
    catch {
        $parentOfFirstCommit = ""
    }
    Write-Debug "parent commit: $parentOfFirstCommit"

    $commit = $commits[0]
    Write-Debug "commit used for last commit: $commit"
    $lastCommit = ($commit -split ";")[1]
    if ($parentOfFirstCommit) {
        # Todo: Read tag compare string from config
        $compareUrl = ("{0}/branchCompare?baseVersion=GC{1}&targetVersion=GC{2}" -f $remoteUrl, $parentOfFirstCommit, $lastCommit)
    }
    else {
        $compareUrl = ""
    }
    Write-Debug "comapre url for commits: $compareUrl"

    $commitsObj = @()
    Write-Debug "iter over all commits"
    foreach ($commit in $commits) {
        if (-not $commit -is [String]) { continue }
        $c = $commit -split ";"
        $d = $c[2] -split ":"
        if ($d.Length -gt 1) {
            if ($d[0] -like "*!") { $breaking = $true } else { $breaking = $false }
            if (($relevant -contains $d[0]) -or $breaking) {
                $commitsObj += @{ Short = $c[0].Trim(); Long = $c[1].Trim(); Type = $d[0].Trim(); Description = $d[1].Trim(); Breaking = $breaking }
            }
        }
        elseif ($IncludeAll -and ($c[2] -notlike "*no_ci*") -and ($c[2] -notlike "*PR *")) {
            $commitsObj += @{ Short = $c[0].Trim(); Long = $c[1].Trim(); Type = "other"; Description = $c[2].Trim(); Breaking = $breaking }
        }
    }
    Write-Debug "iterated over all commits"
    if ($range) {
        $breakingChanges = Get-Commits $range "--grep `"breaking change`" -i"
        foreach ($commit in $breakingChanges ) {
            if (-not $commit -is [String]) { continue }
            $c = $commit -split ";"
            if (($d[0] -notlike "*!") -and ($c[2] -notlike "*no_ci*") -and ($c[2] -notlike "*PR *")) {
                $commitsObj += @{Short = $c[0].Trim(); Long = $c[1].Trim(); Description = $c[2].Trim(); Breaking = $true; BreakingMsg = $c[3].Trim() }
            }
        }
    }

    $changelog = @()
    if ($compareUrl) {
        $changelog += ("## [v{0}]({1}) ({2})`n" -f $version, $compareUrl, (Get-Date -Format "yyyy-MM-dd"))
    }
    else {
        $changelog += ("## v{0} ({1})`n" -f $version, (Get-Date -Format "yyyy-MM-dd"))
    }
    Write-Debug "build changelog with heading: $changlog"


    if ($commitsObj.Count -gt 0) {
        Write-Debug "build changelog with $($commitsObj.Count) objects"

        $headings = @{
            feat     = "Features";
            fix      = "Bug Fixes";
            revert   = "Reverts";
            breaking = "BREAKING CHANGES";
            other    = "Other Commits"
        }

        $features = @()
        $fix = @()
        $reverts = @()
        $breaking = @()
        $other = @()
        foreach ($entry in $commitsObj) {
            $msg = ("* {0} ([{1}]({2}/{3}))" -f $entry.Description, $entry.Short, $commitUrl, $entry.Long)
            switch ($entry.Type) {
                "feat" { $features += $msg }
                "fix" { $fix += $msg }
                "revert" { $reverts += $msg }
                Default { $other += $msg }
            }
            if ($entry.Breaking) {
                if ($entry.BreakingMsg) {
                    $msg = ("* {0} ([{1}]({2}/{3}))`n{4}" -f $entry.Description, $entry.Short, $commitUrl, $entry.Long, $entry.BreakingMsg)
                }
                else {
                    $msg = ("* {0} ([{1}]({2}/{3}))" -f $entry.Description, $entry.Short, $commitUrl, $entry.Long)
                }
                $breaking += $msg
            }
        }
        if ($features.Length -gt 0) {
            $changelog += (Write-ToChangelog $headings.feat $features)
        }
        if ($fix.Length -gt 0) {
            $changelog += (Write-ToChangelog $headings.fix $fix)
        }
        if ($reverts.Length -gt 0) {
            $changelog += (Write-ToChangelog $headings.reverts $reverts)
        }
        if ($breaking.Length -gt 0) {
            $changelog += (Write-ToChangelog $headings.breaking $breaking)
        }
        if ($other.Length -gt 0) {
            $changelog += (Write-ToChangelog $headings.other $other)
        }
    }
    elseif ($IncludeAll) {
        $commitsObj | ForEach-Object { $changelog += "* {0} ([{1}]({2}/{3}))" -f $_.Description, $_.Short, $commitUrl, $_.Long }
        $changelog += "`n"
    }
    $oldLog = @()
    if (Test-Path $changelogFile) {
        $oldLog = Get-Content $changelogFile
    }
    $changelog += $oldLog
    Set-Content -Path $changelogFile -Value $changelog -Encoding utf8
    Write-Host "Generated changelog at '$changelogFile'"
    return $changelogFile
}

function Write-ToChangelog {
    [CmdletBinding()]
    param (
        [Parameter(Position = 0)][string]$heading,
        [Parameter(Position = 1)][string[]]$msgs
    )
    $ce = @()
    $ce += "### $heading`n"
    $msgs | ForEach-Object { $ce += $_ }
    $ce += "`n"
    return $ce
}

function Get-Commits {
    [CmdletBinding()]
    param (
        [Parameter()]
        [string]
        $range,
        [Parameter()]
        [string[]]
        $arguments
    )
    $callArguments = @()
    $callArguments += "--no-pager"
    $callArguments += "log"
    $callArguments += "--no-merges"
    $callArguments += "--pretty=`"%h;%H;%s`""
    $callArguments += if ($range) { $range }
    if ($arguments) { $arguments | ForEach-Object { $callArguments += $_.ToString() } }

    # $commits = Start-Command "git" $callArguments -Wait

    if ($range) {
        $commits = & git --no-pager log --no-merges --pretty="%h;%H;%s" $range.toString()
    }
    else {
        $commits = & git --no-pager log --no-merges --pretty="%h;%H;%s"
    }
    return $commits
}

function Assert-IsRepository {
    [CmdletBinding()]
    param (
        [Parameter()]
        [string]
        $Repository
    )
    if(-not (Test-Path (Join-Path $Repository ".git"))) { throw "Script must be executed inside a git repository." }
}