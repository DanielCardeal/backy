use core::fmt;

/// Erros que podem ocorrer durante a execução do programa
pub enum BackyError {
    /// Não foi passado um comando par o programa
    NoCommand,
    /// O comando passado não existe
    BadCommand(String),
}

impl BackyError{
    fn get_err_msg(&self) -> String {
        match self {
            BackyError::NoCommand => "sem comandos para executar".to_string(),
            BackyError::BadCommand(cmd) => format!("o comando '{}' não existe", cmd),
        }
    }
}

impl fmt::Display for BackyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let msg = self.get_err_msg();
        write!(f, "Erro: {}. Tente `backy help` para mais informações.", msg)
    }
}
