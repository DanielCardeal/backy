use crate::logging::{error, log};

// #######################
//   Definições públicas
// #######################
/// Um erro encontrado durante a execução do programa.
pub trait BackyError {
    /// Mostra a mensagem de erro relevante para o usuário
    fn display(&self) {
        let msg = self.get_err_msg();
        error!(msg);
    }

    /// Devolve a mensagem de erro a ser mostrada para o usuário
    fn get_err_msg(&self) -> String;
}

/// Type alias para simplificar a devolução de possíveis erros de execução no
/// programa
pub type BackyResult<T> = Result<T, Box<dyn BackyError>>;
