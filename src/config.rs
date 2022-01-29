use crate::error::BackyError;
use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

// #######################
//   Definições públicas
// #######################
/// Representa as configurações do usuário antes da manipulação e transformação em Config.
#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    /// Caminho para a base do backup do usuário. É inicializado como $HOME caso
    /// nenhum valor seja associado.
    pub backup_root: Option<PathBuf>,
    /// Lista dos arquivos/diretórios que devem ser ignorados pelo backup.
    pub exclude_files: Option<Vec<String>>,
    /// Local do disco onde devem ser armazenados os backups incrementais.
    pub archive_path: String,
    /// Número máximo de dias que um backup deve armazenado pelo programa.
    pub remove_older_than: i64,
    /// Nome do remote que deve ser usado pelo rclone para sincronizar os arquivos
    pub rclone_remote: String,
}

/// Carrega o arquivo de configuração do usuário e devolve uma struct com os
/// valores já settados.
pub fn load() -> Result<Config, Box<dyn BackyError>> {
    // Carrega arquivo de configuração
    let config_str = read_config()?;
    let mut config: Config = match toml::from_str(&config_str) {
        Ok(c) => c,
        Err(err) => return Err(Box::new(ErrBadConfigFormat { err })),
    };
    // Tenta inferir o diretório $HOME como base do backup caso nenhum valor
    // seja passado
    config.backup_root = config.backup_root.or_else(dirs::home_dir);

    Ok(config)
}

// #######################
//   Definições públicas
// #######################
/// Lê o arquivo de configuração do usuário e o devolve como uma string
fn read_config() -> Result<String, Box<dyn BackyError>> {
    // Encontra o path para o arquivo de configuração
    let mut config_path = match dirs::config_dir() {
        Some(d) => d,
        None => return Err(Box::new(ErrNoConfigDir)),
    };
    config_path.push("backy/config.toml");
    // Lê a config para uma string
    match fs::read_to_string(config_path) {
        Ok(f) => Ok(f),
        _ => return Err(Box::new(ErrNoConfigFile)),
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
