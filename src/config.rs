use crate::error::BackyError;
use std::fs;

use serde::{Deserialize, Serialize};

// #######################
//    Structs e lógica
// #######################
/// Representa as configurações do usuário antes da manipulação e transformação em Config.
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    /// Lista dos arquivos que devem ser armazenados pelo backup.
    pub files: Vec<String>,
    /// Local do disco onde devem ser armazenados os backups incrementais.
    pub archive_path: String,
    /// Número máximo de dias que um backup deve armazenado pelo programa.
    pub remove_older_than: i64,
    /// Nome do remote que deve ser usado pelo rclone para sincronizar os arquivos
    pub rclone_remote: String,
}

pub fn load() -> Result<Config, Box<dyn BackyError>> {
    let mut config_path = match dirs::config_dir() {
        Some(d) => d,
        None => return Err(Box::new(ErrNoConfigDir)),
    };
    config_path.push("backy/config.toml");
    let config_file = match fs::read_to_string(config_path) {
        Ok(f) => f,
        _ => return Err(Box::new(ErrNoConfigFile)),
    };
    match toml::from_str(&config_file) {
        Ok(c) => Ok(c),
        Err(err) => Err(Box::new(ErrBadConfigFormat { err })),
    }
}

// #######################
//         Erros
// #######################
/// Erro lançado quando o arquivo de configuração não pode ser encontrado
struct ErrNoConfigFile;
impl BackyError for ErrNoConfigFile {
    fn get_err_msg(&self) -> String {
        "unable to open the configuration file (maybe the file doesn't exist?)".into()
    }
}

/// Erro lançado quando o diretório de configuração não existe
struct ErrNoConfigDir;
impl BackyError for ErrNoConfigDir {
    fn get_err_msg(&self) -> String {
        "unable to open the configuration directory".into()
    }
}

/// Erro lançado quando o arquivo de configuração está mal formado, ou seja, não
/// tem as configurações necessárias para a execução do programa
struct ErrBadConfigFormat {
    err: toml::de::Error,
}
impl BackyError for ErrBadConfigFormat {
    fn get_err_msg(&self) -> String {
        format!("unable to parse config:\n{}", self.err)
    }
}
