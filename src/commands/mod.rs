mod clean;
mod help;
mod remote;
mod update;

use crate::{
    config::Config,
    error::{BackyError, BackyResult},
};
use std::{
    io,
    process::{self, Stdio},
};

// #######################
//        Comandos
// #######################
/// Comandos (ou modos de operação) que o programa pode ser executado.
pub trait BackyCommand {
    /// Executa um comando
    fn execute(&self, config: Config) -> BackyResult;
}

/// Cria e devolve o comando correspondente à lista argumentos
pub fn from_args(args: &[String]) -> Result<Box<dyn BackyCommand>, Box<dyn BackyError>> {
    if args.len() <= 1 {
        return Err(Box::new(ErrNoCommand));
    }
    match args[1].as_str() {
        "help" => Ok(Box::new(help::CmdHelp)),
        "clean" => Ok(Box::new(clean::CmdClean)),
        "update" => Ok(Box::new(update::CmdUpdate)),
        "remote" => Ok(Box::new(remote::CmdRemote)),
        cmd => Err(Box::new(ErrBadCommand {
            cmd: cmd.to_string(),
        })),
    }
}

// #######################
//   Funções auxiliares
// #######################
/// Checa se o usuário tem o programa `rsync` instalado
fn user_has_rsync() -> bool {
    process::Command::new("rsync")
        .arg("--version")
        .stdout(Stdio::null())
        .status()
        .is_ok()
}

/// Checa se o usuário tem o programa `rclone` instalado
fn user_has_rclone() -> bool {
    process::Command::new("rclone")
        .arg("--version")
        .stdout(Stdio::null())
        .status()
        .is_ok()
}

// #######################
//         Erros
// #######################
/// Erro lançado quando o usuário não fornece um comando para o programa
/// executar
struct ErrNoCommand;
impl BackyError for ErrNoCommand {
    fn get_err_msg(&self) -> String {
        "no command to execute. Try `backy help` for aditional information.".into()
    }
}

/// Erro lançado quando o comando que usuário deseja executar não existe
struct ErrBadCommand {
    cmd: String,
}
impl BackyError for ErrBadCommand {
    fn get_err_msg(&self) -> String {
        format!("command '{}' doesn't exist", self.cmd).into()
    }
}

/// Erro lançado quando não é possível encontrar o executável do rsync no PATH
/// do usuário
struct ErrNoRsync;
impl BackyError for ErrNoRsync {
    fn get_err_msg(&self) -> String {
        "unable to find `rsync` executable".into()
    }
}

/// Erro lançado quando não é possível encontrar o executável do rsync no PATH
/// do usuário
struct ErrNoRclone;
impl BackyError for ErrNoRclone {
    fn get_err_msg(&self) -> String {
        "unable to find `rclone` executable".into()
    }
}
