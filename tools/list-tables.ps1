$path = "D:/workspace/git/rust-nrcs/crates/orm/schema_analysis.json"
if (!(Test-Path $path)) { Write-Error "File not found: $path"; exit 1 }
$schema = Get-Content $path | ConvertFrom-Json
$schema.tables.Keys | Sort-Object | ForEach-Object { Write-Host $_ }
