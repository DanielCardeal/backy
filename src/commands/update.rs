use chrono::Utc;

use super::{user_has_rsync, BackyCommand, ErrNoRsync};

use crate::{
    config::Config,
    error::{BackyError, BackyResult},
    logging::{info, log},
};

use std::{fs, io, os::unix::fs::symlink, path::PathBuf, process};

// #######################
//   Definições públicas
// #######################
pub struct CmdUpdate;
impl BackyCommand for CmdUpdate {
    fn execute(&self, config: Config) -> BackyResult {
        if !user_has_rsync() {
            return Err(Box::new(ErrNoRsync));
        }
        if let None = &config.backup_root {
            return Err(Box::new(ErrNoBackupRoot));
        }
        let backup_root_str = gen_backup_root_str(&config.backup_root.unwrap())?;

        // Cria o diretório do backup de hoje
        let backup_dir = create_backup_dir(&config.archive_path)?;
        let latest_link = format!("{}/latest", &config.archive_path);
        let latest_arg = format!("--link-dest={}", latest_link);

        // Faz o backup dos backups, excluíndo arquivos caso necessário
        info!("Backing up '{}'.", &backup_root_str);
        if let Some(exclude_files) = &config.exclude_files {
            let exclude_arg = gen_exclude_arg(&exclude_files);
            process::Command::new("rsync")
                .current_dir(&backup_dir)
                .arg(&backup_root_str)
                .args(["-az", "--delete", "--copy-links", &latest_arg])
                .args(&exclude_arg)
                .arg(".")
                .status()
                .unwrap();
        } else {
            process::Command::new("rsync")
                .current_dir(&backup_dir)
                .arg(&backup_root_str)
                .args(["-az", "--delete", "--copy-links", &latest_arg])
                .arg(".")
                .status()
                .unwrap();
        }

        // Recria o link simbólico para latest
        info!("Updating `latest` link.");
        fs::remove_file(&latest_link).ok();
        if let Err(err) = symlink(&backup_dir, &latest_link) {
            return Err(Box::new(ErrSymCreationFailed { err }));
        };

        Ok(())
    }
}

// #######################
//   Definições privadas
// #######################
/// Cria um diretório para o backup.
fn create_backup_dir(archive_path: &str) -> Result<PathBuf, Box<dyn BackyError>> {
    let today = Utc::today().format("%Y%m%d/").to_string();
    let mut backup_dir = PathBuf::from(archive_path);
    backup_dir.push(&today);
    match fs::create_dir_all(&backup_dir) {
        Err(err) => Err(Box::new(ErrSymCreationFailed { err })),
        _ => Ok(backup_dir),
    }
}

/// Gera a string que representa o diretório base do backup
fn gen_backup_root_str(backup_root: &PathBuf) -> Result<String, Box<dyn BackyError>> {
    let mut backup_root = backup_root.clone();
    if !backup_root.is_dir() {
        return Err(Box::new(ErrBackupRootNotDir));
    }
    // NOTE: isso garante que o path terá um '/' no final, o que impede que o
    // impede que o rsync crie um subdiretório acima do backup
    backup_root.push(PathBuf::from(""));
    return Ok(format!("{}", backup_root.display()));
}

/// Gera diretivas --exclude para os arquivos passados pelo usuário
fn gen_exclude_arg<'a>(exclude_files: &'a[String]) -> Vec<&'a str> {
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
/// Erro lançado quando não foi possível inferir/não foi fornecido um diretório
/// base para o backup
struct ErrNoBackupRoot;
impl BackyError for ErrNoBackupRoot {
    fn get_err_msg(&self) -> String {
        "unable to infer a root for the backup. Please set the value `backup_root` on your config file.".into()
    }
}

/// Erro lançado quando o diretório base do backup não é um diretório
struct ErrBackupRootNotDir;
impl BackyError for ErrBackupRootNotDir {
    fn get_err_msg(&self) -> String {
        "the backup_root is not a directory.".into()
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
