$libPath = "D:/workspace/git/rust-nrcs/crates/orm/src/lib.rs"
Add-Content -Path $libPath -Value "`n#[cfg(test)]`nmod generated_model_tests;`n"
Write-Host "Added test module import to lib.rs"
