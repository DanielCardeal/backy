use super::{
    user_has_rclone, BackyCommand, ErrBadRemoteName, ErrCompressionFailed, ErrInacessibleRemote,
    ErrNoRclone, ErrSendRemoteFail,
};

use crate::{config::Config, error::BackyResult};

use chrono::{Datelike, Utc};
use std::{
    process::{self, Stdio},
    str,
};
use tempfile::tempdir;

pub struct CmdRemote;
impl BackyCommand for CmdRemote {
    /// Atualiza o drive externo com a versão atual do backup
    fn execute(&self, config: Config) -> BackyResult {
        if !user_has_rclone() {
            return Err(Box::new(ErrNoRclone));
        }
        if !rclone_valid_remote(&config.rclone_remote) {
            return Err(Box::new(ErrBadRemoteName));
        }

        // Testa conexão com o remote do usuário
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

/// Checa se o remote passado pelo usuário é um remote válido
fn rclone_valid_remote(rclone_remote: &str) -> bool {
    let listremotes_output = process::Command::new("rclone")
        .arg("listremotes")
        .output()
        .unwrap();
    let remotes = str::from_utf8(&listremotes_output.stdout).unwrap();
    return remotes.lines().any(|remote| rclone_remote.eq(remote));
}
