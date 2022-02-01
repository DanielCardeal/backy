use chrono::Utc;

use super::{user_has_rsync, BackyCommand, ErrNoRsync};

use crate::{
    config::{BackupDescription, Config},
    error::{BackyError, BackyResult},
    logging::{info, log},
};

use std::{fs, io, os::unix::fs::symlink, path::PathBuf, process, sync::Arc, thread};

// #######################
//   Definições públicas
// #######################
pub struct CmdUpdate;
impl BackyCommand for CmdUpdate {
    fn execute(&self, config: Config) -> BackyResult<()> {
        if !user_has_rsync() {
            return Err(Box::new(ErrNoRsync));
        }
        // Cria o diretório do backup de hoje
        let backup_dir = Arc::new(create_backup_dir(&config.archive_path)?);
        let latest_link = {
            let mut latest_link = config.archive_path.clone();
            latest_link.push("latest");
            Arc::new(latest_link)
        };

        // Executa backups assíncronamente
        let mut backup_handles = Vec::new();
        for (name, desc) in config.backups {
            let backup_dir = backup_dir.clone();
            let latest_link = latest_link.clone();
            let backup_task =
                thread::spawn(move || create_named_backup(&backup_dir, &latest_link, &name, &desc));
            backup_handles.push(backup_task);
        }

        // Coleta erros caso encontre algum
        for backup in backup_handles {
            let result = backup.join().unwrap();
            result?;
        }

        // Recria o link simbólico para latest
        info!("Updating `latest` link.");
        fs::remove_file(&*latest_link).ok();
        if let Err(err) = symlink(&*backup_dir, &*latest_link) {
            return Err(Box::new(ErrLatestUpdate { err }));
        };

        Ok(())
    }
}

// #######################
//   Definições privadas
// #######################
/// Cria um diretório para o backup.
fn create_backup_dir(archive_path: &PathBuf) -> BackyResult<PathBuf> {
    let today = Utc::today().format("%Y%m%d/").to_string();
    let mut backup_dir = archive_path.clone();
    backup_dir.push(&today);
    match fs::create_dir_all(&backup_dir) {
        Err(err) => Err(Box::new(ErrArchiveCreationFailed { err })),
        _ => Ok(backup_dir),
    }
}

/// Gera a string que representa o diretório base do backup
fn gen_backup_root_str(backup_root: &PathBuf) -> BackyResult<String> {
    let mut backup_root = backup_root.clone();
    if !backup_root.is_dir() {
        return Err(Box::new(ErrBackupRootNotDir));
    }
    // NOTE: isso garante que o path terá um '/' no final, o que impede que o
    // impede que o rsync crie um subdiretório acima do backup
    backup_root.push(PathBuf::from(""));
    return Ok(format!("{}", backup_root.display()));
}

/// Usa o rsync para gerar um backup nomeado na pasta de backups.
fn create_named_backup(
    backup_dir: &PathBuf,
    latest_link: &PathBuf,
    name: &String,
    desc: &BackupDescription,
) -> BackyResult<()> {
    let backup_root_str = gen_backup_root_str(&desc.backup_root)?;
    let mut latest_link = latest_link.clone();
    latest_link.push(name);

    // Cria o comando `rsync` para o backup dos arquivos selecionados
    let mut rsync_command = process::Command::new("rsync");
    rsync_command
        .current_dir(&backup_dir)
        .arg(&backup_root_str)
        .args(["-az", "--delete"])
        .arg("--link-dest")
        .arg(latest_link)
        .arg(&name);

    if let Some(exclude_files) = &desc.exclude_files {
        let exclude_arg = gen_exclude_arg(&exclude_files);
        rsync_command.args(exclude_arg);
    }

    // Executa o backupaa
    info!("Creating '{}' backup.", &name);
    let rsync_status = rsync_command.status().unwrap();
    if !rsync_status.success() {
        return Err(Box::new(ErrRsyncFail));
    }

    Ok(())
}

/// Gera diretivas --exclude para os arquivos passados pelo usuário
fn gen_exclude_arg<'a>(exclude_files: &'a [String]) -> Vec<&'a str> {
    let mut exclude_arg = Vec::new();
    for file in exclude_files {
        exclude_arg.push("--exclude");
        exclude_arg.push(file);
    }
    return exclude_arg;
}

// #######################
//         Erros
// #######################
/// Erro lançado quando o diretório base do backup não é um diretório
struct ErrBackupRootNotDir;
impl BackyError for ErrBackupRootNotDir {
    fn get_err_msg(&self) -> String {
        "the backup_root is not a directory.".into()
    }
}

/// Erro lançado quando não é possível criar o diretório local de arquivos de backup
struct ErrArchiveCreationFailed {
    err: io::Error,
}
impl BackyError for ErrArchiveCreationFailed {
    fn get_err_msg(&self) -> String {
        format!("unable to create backup archive directory:\n{}", &self.err)
    }
}

/// Erro lançado quando não é possível criar o link simbólico para o backup mais
/// atual
struct ErrLatestUpdate {
    err: io::Error,
}
impl BackyError for ErrLatestUpdate {
    fn get_err_msg(&self) -> String {
        format!("unable to update `latest` symlink:\n{}", self.err)
    }
}

/// Erro lançado quando algum problema é encontrado na execução do comando rsync
struct ErrRsyncFail;
impl BackyError for ErrRsyncFail {
    fn get_err_msg(&self) -> String {
        "rsync failed to create user backup. Error description can be found above.".into()
    }
}
