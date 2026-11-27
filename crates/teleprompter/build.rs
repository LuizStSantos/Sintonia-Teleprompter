//! No Windows, embute no `.exe` o ícone do Sintonia e os dados do programa
//! (nome, versão, empresa) que aparecem no Explorador de Arquivos e na barra
//! de tarefas. Nos outros sistemas não faz nada.

use std::{env, fs, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=assets/icones/sintonia.ico");
    println!("cargo:rerun-if-changed=build.rs");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }
    let versao = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".into());
    let numeros: Vec<u32> = versao
        .split(['.', '-'])
        .take(3)
        .map(|p| p.parse().unwrap_or(0))
        .chain(std::iter::repeat(0))
        .take(4)
        .collect();
    let virgulas = numeros.iter().map(u32::to_string).collect::<Vec<_>>().join(",");
    let icone = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("assets/icones/sintonia.ico")
        .to_string_lossy()
        .replace('\\', "/");
    let rc = format!(
        r#"#pragma code_page(65001)
1 ICON "{icone}"
1 VERSIONINFO
FILEVERSION {virgulas}
PRODUCTVERSION {virgulas}
FILEOS 0x40004
FILETYPE 0x1
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "041604B0"
    BEGIN
      VALUE "CompanyName", "Sintonia"
      VALUE "FileDescription", "Sintonia Teleprompter"
      VALUE "FileVersion", "{versao}"
      VALUE "InternalName", "sintonia-teleprompter"
      VALUE "OriginalFilename", "Sintonia Teleprompter.exe"
      VALUE "ProductName", "Sintonia Teleprompter"
      VALUE "ProductVersion", "{versao}"
      VALUE "LegalCopyright", "Sintonia"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x0416, 1200
  END
END
"#
    );
    let saida = PathBuf::from(env::var("OUT_DIR").unwrap()).join("sintonia-teleprompter.rc");
    fs::write(&saida, rc).expect("gravar .rc");
    embed_resource::compile(&saida, embed_resource::NONE)
        .manifest_optional()
        .expect("embutir ícone e versão no .exe");
}
