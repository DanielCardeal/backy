use super::{user_has_rclone, BackyCommand, ErrNoRclone};

use crate::{
    config::Config,
    error::{BackyError, BackyResult},
    logging::{info, log},
};

use chrono::{Datelike, Utc};
use std::{
    process::{self, Stdio},
    str,
};
use tempfile::tempdir;

// #######################
//   Definições públicas
// #######################
pub struct CmdRemote;
impl BackyCommand for CmdRemote {
    /// Atualiza o drive externo com a versão atual do backup
    fn execute(&self, config: Config) -> BackyResult<()> {
        if !user_has_rclone() {
            return Err(Box::new(ErrNoRclone));
        }
        if !rclone_valid_remote(&config.rclone_remote) {
            return Err(Box::new(ErrBadRemoteName));
        }

        // Testa conexão com o remote do usuário
        info!(
            "Testing conection with remote drive `{}`.",
            &config.rclone_remote
        );
        if process::Command::new("rclone")
            .stderr(Stdio::null())
            .current_dir(&config.archive_path)
            .args(["sync", "--dry-run", "latest/", &config.rclone_remote])
            .status()
            .is_err()
        {
            return Err(Box::new(ErrInacessibleRemote));
        }

        // Comprime o backup
        info!("Compressing backup data");
        let today = Utc::today();
        let backup_file_name = format!(
            "backy_{}-{}-{}.tar.gz",
            &today.year(),
            &today.month(),
            &today.day()
        );
        let temporary_dir = tempdir().unwrap();
        let compressed_filepath = temporary_dir.path().join(backup_file_name);
        if process::Command::new("tar")
            .current_dir(&config.archive_path)
            .stdout(Stdio::null())
            .arg("-vczpf")
            .arg(&compressed_filepath)
            .arg("./")
            .status()
            .is_err()
        {
            return Err(Box::new(ErrCompressionFailed));
        }

        // Sincroniza o backup com o remote
        info!("Syncing data with remote");
        if process::Command::new("rclone")
            .arg("sync")
            .arg("--progress")
            .arg(&compressed_filepath)
            .arg(&config.rclone_remote)
            .status()
            .is_err()
        {
            return Err(Box::new(ErrSendRemoteFail));
        }

        Ok(())
    }
}

// #######################
//   Definições privadas
// #######################
/// Checa se o remote passado pelo usuário é um remote válido
fn rclone_valid_remote(rclone_remote: &str) -> bool {
    let listremotes_output = process::Command::new("rclone")
        .arg("listremotes")
        .output()
        .unwrap();
    let remotes = str::from_utf8(&listremotes_output.stdout).unwrap();
    return remotes.lines().any(|remote| rclone_remote.eq(remote));
}

// #######################
//         Erros
// #######################
/// Erro lançado quando não é possível enviar os arquivos de backup para o drive
/// remoto
struct ErrSendRemoteFail;
impl BackyError for ErrSendRemoteFail {
    fn get_err_msg(&self) -> String {
        "unable to send compressed backup to rclone remote".into()
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
