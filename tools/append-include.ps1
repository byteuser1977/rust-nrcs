$path = "D:/workspace/git/rust-nrcs/crates/orm/src/models.rs"
Add-Content -Path $path -Value "`n// Auto-generated models (DB-02)`ninclude!(`"generated_models.rs`");`n"
Write-Host "Appended include to $path"
