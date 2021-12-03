use core::fmt;
use std::io;

/// Erros que podem ocorrer durante a execução do programa
pub enum BackyError {
    /// Não foi passado um comando par o programa
    NoCommand,
    /// O comando passado não existe
    BadCommand(String),

    /// Não foi possível abrir o arquivo de configuração
    NoConfig,
    /// Não foi possível encontrar o diretório de configuração
    NoConfigDir,
    /// O arquivo de configuração contém erros de formatação ou typos
    BadConfigFormat(toml::de::Error),

    /// Não foi possível criar o diretório de backup
    NoArchiveDir(io::Error),
    /// Alguns dos arquivos de backup não existem no sistema
    BadFiles(Vec<String>),
    /// Não foi possível criar o link simbólico nos archives
    SymCreationFailed(io::Error),

    /// Não foi possível encontrar o comando rsync no PATH do usuário
    NoRsync,
}

impl BackyError {
    fn get_err_msg(&self) -> String {
        match self {
            BackyError::NoCommand => {
                "no command to execute. Try `backy help` for aditional information.".to_string()
            }
            BackyError::BadCommand(cmd) => format!("command '{}' doesn't exist", cmd),
            BackyError::NoConfig => {
                "unable to open the configuration file (maybe the file doesn't exist?)".to_string()
            }
            BackyError::NoConfigDir => "unable to open the configuration directory".to_string(),
            BackyError::BadConfigFormat(err) => {
                format!("unable to parse config:\n{}", err).to_string()
            }
            BackyError::NoArchiveDir(err) => {
                format!("unable to create backup dir:\n{}", err)
            }
            BackyError::BadFiles(files) => {
                let mut msg = String::from("unable to find the following files:");
                for file in files {
                    msg.push('\n');
                    msg.push_str(file);
                }
                msg
            }
            BackyError::SymCreationFailed(err) => {
                format!("unable to create `latest` symlink:\n{}", err)
            }
            BackyError::NoRsync => "unable to find `rsync` executable".to_string(),
        }
    }
}

impl fmt::Display for BackyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = self.get_err_msg();
        write!(f, "Error: {}.", msg)
    }
}
