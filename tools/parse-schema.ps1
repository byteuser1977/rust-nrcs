# PowerShell schema parser for H2 DDL

$SqlFile = "D:/workspace/git/nrcs/nrcs-sql/src/main/resources/sql-scripts-h2/0.sql"
$OutputFile = "D:/workspace/git/rust-nrcs/crates/orm/schema_analysis.json"

# Read SQL content
$sql = Get-Content -Raw $SqlFile
$sql = $sql -replace "`r`n", "`n" -replace "`r", "`n"

$tables = @{}

# Split SQL by CREATE TABLE statements (keep delimiter)
$blocks = $sql -split "(?=CREATE\s+TABLE\s+IF\s+NOT\s+EXISTS)"

foreach ($block in $blocks) {
    $trim = $block.Trim()
    if ($trim -eq "") { continue }

    # Extract table name
    if ($trim -notmatch 'CREATE\s+TABLE\s+IF\s+NOT\s+EXISTS\s+(\w+)') { continue }
    $tableName = $matches[1].ToUpper()

    # Extract inner parentheses content
    $startIdx = $trim.IndexOf('(')
    if ($startIdx -lt 0) { continue }
    $parenCount = 0
    $endIdx = -1
    for ($i = $startIdx; $i -lt $trim.Length; $i++) {
        if ($trim[$i] -eq '(') { $parenCount++ }
        elseif ($trim[$i] -eq ')') { $parenCount--; if ($parenCount -eq 0) { $endIdx = $i; break } }
    }
    if ($endIdx -lt 0) { continue }

    $inner = $trim.Substring($startIdx + 1, $endIdx - $startIdx - 1)

    $columns = @()
    $pkColumns = @()
    $fks = @()

    # Split inner content by commas at depth 0
    $parts = @()
    $current = ""
    $depth = 0
    for ($i=0; $i -lt $inner.Length; $i++) {
        $ch = $inner[$i]
        if ($ch -eq '(') { $depth++ }
        elseif ($ch -eq ')') { $depth-- }
        if ($ch -eq ',' -and $depth -eq 0) {
            $parts += $current.Trim()
            $current = ""
        } else {
            $current += $ch
        }
    }
    if ($current.Trim() -ne "") { $parts += $current.Trim() }

    foreach ($part in $parts) {
        $p = $part.Trim()
        if ($p -eq "") { continue }

        # PRIMARY KEY constraint
        if ($p -match '^PRIMARY\s+KEY\s*\(([^)]+)\)') {
            $pkColumns = $matches[1] -split ',' | ForEach-Object { $_.Trim() }
            continue
        }

        # FOREIGN KEY constraint (inline)
        if ($p -match '^CONSTRAINT\s+\w+\s+FOREIGN\s+KEY\s*\(([^)]+)\)\s+REFERENCES\s+(\w+)\s*\(([^)]+)\)(?:\s+ON\s+DELETE\s+(\w+))?') {
            $fkCols = $matches[1] -split ',' | ForEach-Object { $_.Trim() }
            $refTable = $matches[2].ToUpper()
            $refCols = $matches[3] -split ',' | ForEach-Object { $_.Trim() }
            $onDelete = if ($matches[4]) { $matches[4] } else { $null }

            for ($i=0; $i -lt $fkCols.Count; $i++) {
                $fk = [PSCustomObject]@{
                    column = $fkCols[$i]
                    referenced_table = $refTable
                    referenced_column = $refCols[$i]
                    on_delete = $onDelete
                }
                $fks += $fk
            }
            continue
        }

        # Column definition
        if ($p -match '^([A-Z_][A-Z0-9_]*)\s+([\w\(\)]+)(.*)$') {
            $colName = $matches[1]
            $dataType = $matches[2]
            $rest = $matches[3].Trim()

            $isNullable = $rest -notmatch 'NOT\s+NULL'
            $isAutoInc = $rest -match 'AUTO_INCREMENT'
            $isPK = $rest -match 'PRIMARY\s+KEY'

            $defaultVal = $null
            if ($rest -match 'DEFAULT\s+([^,\s]+)') {
                $defaultVal = $matches[1]
            }

            $col = [PSCustomObject]@{
                name = $colName
                data_type = $dataType
                is_nullable = $isNullable
                default_value = $defaultVal
                is_auto_increment = $isAutoInc
                is_primary_key = $isPK
            }
            $columns += $col
        }
    }

    # Mark primary key columns
    foreach ($col in $columns) {
        if ($pkColumns -contains $col.name) {
            $col.is_primary_key = $true
        }
    }

    $table = [PSCustomObject]@{
        name = $tableName
        columns = $columns
        primary_key = $pkColumns
        unique_constraints = @()
        indexes = @()
        foreign_keys = $fks
    }
    $tables[$tableName] = $table
}

# Parse standalone CREATE INDEX and CREATE UNIQUE INDEX statements (outside CREATE TABLE)
$idxRe = [regex]::new('CREATE\s+INDEX\s+IF\s+NOT\s+EXISTS\s+(\w+)\s+ON\s+(\w+)\s*\(([^)]+)\)', [System.Text.RegularExpressions.RegexOptions]::IgnoreCase)
$uniqueIdxRe = [regex]::new('CREATE\s+UNIQUE\s+INDEX\s+IF\s+NOT\s+EXISTS\s+(\w+)\s+ON\s+(\w+)\s*\(([^)]+)\)', [System.Text.RegularExpressions.RegexOptions]::IgnoreCase)

foreach ($match in $idxRe.Matches($sql)) {
    $idxName = $match.Groups[1].Value
    $tbl = $match.Groups[2].Value.ToUpper()
    $cols = $match.Groups[3].Value -split ',' | ForEach-Object { $_.Trim() }
    if ($tables.ContainsKey($tbl)) {
        $idx = [PSCustomObject]@{ name=$idxName; columns=$cols; is_unique=$false }
        $tables[$tbl].indexes += $idx
    }
}
foreach ($match in $uniqueIdxRe.Matches($sql)) {
    $idxName = $match.Groups[1].Value
    $tbl = $match.Groups[2].Value.ToUpper()
    $cols = $match.Groups[3].Value -split ',' | ForEach-Object { $_.Trim() }
    if ($tables.ContainsKey($tbl)) {
        $idx = [PSCustomObject]@{ name=$idxName; columns=$cols; is_unique=$true }
        $tables[$tbl].indexes += $idx
    }
}

# Build schema and output
$schema = [PSCustomObject]@{
    tables = $tables
    total_tables = $tables.Count
}

$json = $schema | ConvertTo-Json -Depth 10
Set-Content -Path $OutputFile -Value $json -Encoding UTF8

Write-Host "Schema analysis complete:"
Write-Host "  Total tables: $($schema.total_tables)"
foreach ($name in $tables.Keys | Sort-Object) {
    $tbl = $tables[$name]
    Write-Host "  - $($name): $($tbl.columns.Count) columns, $($tbl.foreign_keys.Count) FKs, $($tbl.indexes.Count) indexes"
}
Write-Host "Output written to: $OutputFile"
