//! Usuário logado e as permissões dele (mesmas regras do servidor).

use serde::Deserialize;
use serde_json::Value;

#[derive(Clone, Debug, Deserialize)]
pub struct Usuario {
    #[allow(dead_code)]
    pub id: String,
    pub username: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub permissions: Value,
    #[serde(default, rename = "nomeCompleto")]
    pub nome_completo: String,
}

impl Usuario {
    pub fn nome(&self) -> &str {
        if self.nome_completo.trim().is_empty() {
            &self.username
        } else {
            &self.nome_completo
        }
    }

    /// Mesma regra do servidor: admin pode tudo; os demais seguem `permissions[colecao]`.
    pub fn pode(&self, colecao: &str, nivel: &str) -> bool {
        if self.role == "admin" {
            return true;
        }
        let p = self
            .permissions
            .get(colecao)
            .and_then(Value::as_str)
            .unwrap_or("none");
        match nivel {
            "view" => p == "view" || p == "edit",
            "edit" => p == "edit",
            _ => false,
        }
    }
}
