# Generate unit tests for all models (insert full test functions)

$SchemaPath = "D:/workspace/git/rust-nrcs/crates/orm/schema_analysis.json"
$OutputPath = "D:/workspace/git/rust-nrcs/crates/orm/src/generated_model_tests.rs"

$schema = Get-Content $SchemaPath -Raw | ConvertFrom-Json

function To-RustTypeName($tableName) {
    $camel = ($tableName -split '_' | ForEach-Object {
        if ($_.Length -eq 0) { return '' }
        $first = $_.Substring(0,1).ToUpper()
        $rest = $_.Substring(1)
        $first + $rest
    }) -join ''
    return $camel + "Model"
}

function RustDefaultValue($col) {
    $type = $col.data_type
    $nullable = $col.is_nullable
    if ($nullable) {
        return "None"
    }
    switch -Wildcard ($type) {
        'bigint' { return "0" }
        'integer' { return "0" }
        'smallint' { return "0" }
        'tinyint' { return "0" }
        'boolean' { return "false" }
        'varchar*' { return "String::new()" }
        'text' { return "String::new()" }
        'binary*' { return "vec![]" }
        'varbinary' { return "vec![]" }
        'blob' { return "vec![]" }
        default { return "String::new()" }
    }
}

$sb = New-Object System.Text.StringBuilder

$null = $sb.AppendLine("// Auto-generated tests for ORM models. DO NOT EDIT MANUALLY.")
$null = $sb.AppendLine("#[cfg(test)]")
$null = $sb.AppendLine("mod generated_model_tests {")
$null = $sb.AppendLine("    use super::*;")
$null = $sb.AppendLine("    use chrono::Utc;")
$null = $sb.AppendLine("    use serde_json::json;")
$null = $sb.AppendLine()

$tablesObject = $schema.tables
$sortedEntries = $tablesObject.PSObject.Properties | Sort-Object Name

foreach ($entry in $sortedEntries) {
    $tblName = $entry.Name
    $table = $entry.Value
    $structName = To-RustTypeName $tblName

    # Skip existing manually written models? We can test all.
    # We'll test each model.

    $null = $sb.AppendLine("    #[test]")
    $null = $sb.AppendLine("    fn test_${tblName.ToLower()}_construction() {")
    $null = $sb.AppendLine("        let _ = $structName {")

    foreach ($col in $table.columns) {
        $fieldName = $col.name.ToLower()
        $default = RustDefaultValue $col
        if ($col.data_type -match 'timestamp|datetime' -or $col.name -match '.*_AT$') {
            $default = "Utc::now()"
        }
        if ($col.data_type -eq 'json' -or $col.data_type -eq 'jsonb') {
            $default = "json!({})"
        }
        $null = $sb.AppendLine("            $fieldName`: $default,")
    }

    $null = $sb.AppendLine("        };")
    $null = $sb.AppendLine("    }")
    $null = $sb.AppendLine()
}

$null = $sb.AppendLine("}")

[System.IO.File]::WriteAllText($OutputPath, $sb.ToString())
Write-Host "Generated tests to $OutputPath"
