# Extract core tables from schema_analysis.json
$SchemaPath = "D:/workspace/git/rust-nrcs/crates/orm/schema_analysis.json"
$OutputPath = "D:/workspace/git/rust-nrcs/.trae/documents/core_tables_structure.json"

$schema = Get-Content $SchemaPath -Raw | ConvertFrom-Json

$coreTables = @("BLOCK", "TRANSACTION", "ACCOUNT", "ACCOUNT_ASSET", "ASSET")

$result = @{}
foreach ($tbl in $coreTables) {
    if ($schema.tables.PSObject.Properties[$tbl]) {
        $result[$tbl] = $schema.tables.PSObject.Properties[$tbl].Value
        Write-Host "Extracted: $tbl"
    } else {
        Write-Host "Warning: $tbl not found in schema"
    }
}

$result | ConvertTo-Json -Depth 100 | Set-Content $OutputPath
Write-Host "Saved to: $OutputPath"
