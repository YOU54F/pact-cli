$pactDir = $pwd.Path

# Install CLI Tools

Write-Host "--> Downloading Latest Pact broker Client binary)"

$latestRelease = Invoke-WebRequest https://github.com/you54f/pact-cli/releases/latest -Headers @{"Accept"="application/json"}
$json = $latestRelease.Content | ConvertFrom-Json
$tag = $json.tag_name
$architecture = [System.Runtime.InteropServices.RuntimeInformation,mscorlib]::OSArchitecture.ToString().ToLower()
if ($architecture -eq "x64") {
    $architecture = "x86_64"
} elseif ($architecture -eq "arm64") {
    $architecture = "aarch64"
} else {
    Write-Host "Unsupported architecture: $architecture"
    exit 1
}
$url = "https://github.com/you54f/pact-cli/releases/download/$tag/pact-cli-$architecture-windows-msvc.exe"


Write-Host "Downloading $url to $pactDir"
$exe = Join-Path $pactDir "pact-cli.exe"
if (Test-Path "$exe") {
  Remove-Item $exe
}

$downloader = new-object System.Net.WebClient
$downloader.DownloadFile($url, $exe)
Write-Host "--> Downloaded pact-cli to $exe"
# Write-Host "--> Setting executable permissions for pact-cli"
# chmod +x $exe
Write-Host "--> Adding pact-cli to path"
$pactBinariesPath = "$pactDir"
$env:PATH += ";$pactBinariesPath"
Write-Host $env:PATH
pact-cli.exe --help