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

impl BackyError{
    fn get_err_msg(&self) -> String {
        match self {
            BackyError::NoCommand => "sem comandos para executar".to_string(),
            BackyError::BadCommand(cmd) => format!("o comando '{}' não existe", cmd),
            BackyError::NoConfig => "não foi possível abrir o arquivo de configuração".to_string(),
            BackyError::NoConfigDir => {
                "não foi possível abrir o diretório de configuração".to_string()
            }
            BackyError::BadConfigFormat(err) => {
                format!("o arquivo de configuração contém o seguinte erro:\n{}", err).to_string()
            }
        }
    }
}

impl fmt::Display for BackyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = self.get_err_msg();
        write!(f, "Erro: {}. Tente `backy help` para mais informações.", msg)
    }
}
