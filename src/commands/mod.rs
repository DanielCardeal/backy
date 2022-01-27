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

/// Erro lançado quando não é possível criar o diretório para arquivar os
/// backups
struct ErrNoArchiveDir {
    err: io::Error,
}
impl BackyError for ErrNoArchiveDir {
    fn get_err_msg(&self) -> String {
        format!("unable to create backup dir:\n{}", self.err)
    }
}

/// Erro lançado quando alguns dos arquivos de backup não existem no sistema
struct ErrBadFiles {
    missing_files: Vec<String>,
}
impl BackyError for ErrBadFiles {
    fn get_err_msg(&self) -> String {
        let mut msg = String::from("unable to find the following files:");
        for file in &self.missing_files {
            msg.push('\n');
            msg.push_str(&file);
        }
        msg
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

/// Erro lançado quando o nome do remote não consta na lista de remotes válidos
/// do `rclone`
struct ErrBadRemoteName;
impl BackyError for ErrBadRemoteName {
    fn get_err_msg(&self) -> String {
        "invalid rclone_remote setting in config. Run `rclone listremotes` for a list of possible values".into()
    }
}

/// Erro lançado quando não é possível criar uma conexão com o remote
struct ErrInacessibleRemote;
impl BackyError for ErrInacessibleRemote {
    fn get_err_msg(&self) -> String {
        "unable to establish connection with remote. Check your internet connection".into()
    }
}

/// Erro lançado quando não é possível comprimir o backup em um arquivo usando o
/// comando tar
struct ErrCompressionFailed;
impl BackyError for ErrCompressionFailed {
    fn get_err_msg(&self) -> String {
        "there was an unexpected error while generating the compressed backup file".into()
    }
}

/// Erro lançado quando não é possível enviar os arquivos de backup para o drive
/// remoto
struct ErrSendRemoteFail;
impl BackyError for ErrSendRemoteFail {
    fn get_err_msg(&self) -> String {
        "unable to send compressed backup to rclone remote".into()
    }
}

/// Erro lançado quando não é possível criar o link simbólico para o backup mais
/// atual
struct ErrSymCreationFailed {
    err: io::Error,
}
impl BackyError for ErrSymCreationFailed {
    fn get_err_msg(&self) -> String {
        format!("unable to create `latest` symlink:\n{}", self.err)
    }
}

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
//   Definições auxiliares
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
