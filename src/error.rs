use core::fmt;

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
        }
    }
}

impl fmt::Display for BackyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = self.get_err_msg();
        write!(f, "Error: {}.", msg)
    }
}
