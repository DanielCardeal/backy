use chrono::Utc;

use super::{user_has_rsync, BackyCommand, ErrNoRsync};

use crate::{
    config::Config,
    error::{BackyError, BackyResult},
    logging::{info, log},
};

use std::{
    fs, io,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    process, thread,
};

// #######################
//   Definições públicas
// #######################
pub struct CmdUpdate;
impl BackyCommand for CmdUpdate {
    fn execute(&self, config: Config) -> BackyResult {
        if !user_has_rsync() {
            return Err(Box::new(ErrNoRsync));
        }

        // Checa se todos os arquivos que devem ser armazenados existem
        let missing_files = inexistent_files(&config.files);
        if missing_files.len() > 0 {
            return Err(Box::new(ErrBadFiles { missing_files }));
        }

        // Cria o diretório do backup de hoje
        let backup_dir = create_backup_dir(&config.archive_path)?;
        let latest_link = format!("{}/latest", &config.archive_path);
        let latest_arg = format!("--link-dest={}", latest_link);

        // Faz o backup (assíncrono) de cada um dos arquivos
        info!("Updating archive files to most recent version.");
        let mut handles = vec![];
        for file in config.files {
            let backup_dir = backup_dir.clone();
            let latest_arg = latest_arg.clone();
            handles.push(thread::spawn(move || {
                info!("Backing up '{}'.", file);
                let _rsync_status = process::Command::new("rsync")
                    .current_dir(backup_dir)
                    .args(["-az", "--delete", &file, &latest_arg, "."])
                    .status()
                    .unwrap();
            }));
        }
        for handle in handles {
            handle.join().unwrap();
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
/// Lista os arquivos de `files` que não existem.
fn inexistent_files(files: &[String]) -> Vec<String> {
    files
        .iter()
        .filter(|f| !Path::new(f).exists())
        .cloned()
        .collect()
}

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

// #######################
//         Erros
// #######################
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
