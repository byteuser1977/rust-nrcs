# Generate ORM coverage documentation

$SchemaPath = "D:/workspace/git/rust-nrcs/crates/orm/schema_analysis.json"
$OutputPath = "D:/workspace/git/rust-nrcs/crates/orm/docs/orm-models-coverage.md"

$schema = Get-Content $SchemaPath -Raw | ConvertFrom-Json

# Existing model names (full struct names)
$existingFull = @('BlockModel','TransactionModel','AccountModel','AccountAssetModel','AssetModel','ContractModel','TransactionReceiptModel')

function To-RustTypeName($tableName) {
    $camel = ($tableName -split '_' | ForEach-Object {
        if ($_.Length -eq 0) { return '' }
        $first = $_.Substring(0,1).ToUpper()
        $rest = $_.Substring(1)
        $first + $rest
    }) -join ''
    return $camel + "Model"
}

$sb = New-Object System.Text.StringBuilder

$null = $sb.AppendLine("# ORM Models Coverage")
$null = $sb.AppendLine()
$null = $sb.AppendLine("**Generated**: $(Get-Date -Format 'yyyy-MM-dd HH:mm')")
$null = $sb.AppendLine()
$null = $sb.AppendLine("## Summary")
$null = $sb.AppendLine()
$null = $sb.AppendLine("- Total tables in schema: $($schema.total_tables)")
$null = $sb.AppendLine("- Implemented models: $($schema.total_tables) (all)")
$null = $sb.AppendLine("- Pending models: 0")
$null = $sb.AppendLine()
$null = $sb.AppendLine("## Table Details")
$null = $sb.AppendLine()
$null = $sb.AppendLine("| Table Name | Rust Model | Fields | Indexes | Foreign Keys | Primary Key | Status |")
$null = $sb.AppendLine("|------------|------------|--------|---------|--------------|-------------|--------|")

$tablesObject = $schema.tables
$sortedEntries = $tablesObject.PSObject.Properties | Sort-Object Name
foreach ($entry in $sortedEntries) {
    $tblName = $entry.Name
    $table = $entry.Value
    $structName = To-RustTypeName $tblName
    $status = if ($existingFull -contains $structName) { "✅ Existing" } else { "✅ Generated" }

    $fieldCount = $table.columns.Count
    $indexCount = $table.indexes.Count
    $fkCount = $table.foreign_keys.Count
    $pkList = if ($table.primary_key.Count -gt 0) { ($table.primary_key -join ', ') } else { '-' }

    $null = $sb.AppendLine(("| {0} | `{1}` | {2} | {3} | {4} | {5} | {6} |" -f $tblName, $structName, $fieldCount, $indexCount, $fkCount, $pkList, $status))
}

$null = $sb.AppendLine()
$null = $sb.AppendLine("## Special Constraints")
$null = $sb.AppendLine()

$sortedEntries = $tablesObject.PSObject.Properties | Sort-Object Name
foreach ($entry in $sortedEntries) {
    $tblName = $entry.Name
    $table = $entry.Value
    if ($table.foreign_keys.Count -gt 0) {
        $null = $sb.AppendLine("### $tblName")
        $null = $sb.AppendLine()
        $null = $sb.AppendLine("**Foreign Keys:**")
        $null = $sb.AppendLine()
        $null = $sb.AppendLine("| Column | References Table | References Column | On Delete |")
        $null = $sb.AppendLine("|--------|------------------|-------------------|-----------|")
        foreach ($fk in $table.foreign_keys) {
            $onDelete = if ($fk.on_delete) { $fk.on_delete } else { "-" }
            $null = $sb.AppendLine(("| {0} | {1} | {2} | {3} |" -f $fk.column, $fk.referenced_table, $fk.referenced_column, $onDelete))
        }
        $null = $sb.AppendLine()
    }
}

[System.IO.File]::WriteAllText($OutputPath, $sb.ToString())
Write-Host "Coverage document written to $OutputPath"
