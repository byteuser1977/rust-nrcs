# Generate unit tests for ORM models

$SchemaPath = "D:/workspace/git/rust-nrcs/crates/orm/schema_analysis.json"
$ModelsFile = "D:/workspace/git/rust-nrcs/crates/orm/src/generated_models.rs"

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

$null = $sb.AppendLine("")
$null = $sb.AppendLine("#[cfg(test)]")
$null = $sb.AppendLine("mod generated_model_tests {")
$null = $sb.AppendLine("    use super::*;")
$null = $sb.AppendLine("    use chrono::Utc;")
$null = $sb.AppendLine("    use serde_json::json;")
$null = $sb.AppendLine()

foreach ($tblName in ($schema.tables.Keys | Sort-Object)) {
    $structName = To-RustTypeName $tblName
    $table = $schema.tables[$tblName]

    # Skip existing manually written models to avoid duplicate tests? But we can test all.
    # We'll generate for all tables.

    $null = $sb.AppendLine("    #[test]")
    $null = $sb.AppendLine("    fn test_${tblName.ToLower()}_construction() {")
    $null = $sb.AppendLine("        let _ = $structName {")

    foreach ($col in $table.columns) {
        $fieldName = $col.name.ToLower()
        $default = RustDefaultValue $col
        # For DateTime<Utc> fields, use Utc::now()
        if ($col.data_type -match 'timestamp|datetime' -or $col.name -match '.*_AT$') {
            $default = "Utc::now()"
        }
        # For JSON fields (type containing 'json'?) Not in schema.
        # For serde_json::Value, use json!({})
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

# Append to models file
Add-Content -Path $ModelsFile -Value $sb.ToString()
Write-Host "Appended tests to generated_models.rs"
