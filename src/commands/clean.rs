use std::{
    fs::{self, DirEntry},
    io,
    path::PathBuf,
    thread,
};

use chrono::{NaiveDate, Utc};

use crate::error::BackyError;

use super::BackyCommand;

/// Remove backups antigos.
pub struct CmdClean;
impl BackyCommand for CmdClean {
    fn execute(&self, config: crate::config::Config) -> crate::error::BackyResult {
        println!(
            "Removing backups older than {} days.",
            config.remove_older_than
        );
        let backup_dir = PathBuf::from(&config.archive_path);
        let backup_list: Vec<DirEntry> = match fs::read_dir(backup_dir) {
            Ok(list) => list,
            Err(err) => return Err(Box::new(ErrNoArchiveDir { err })),
        }
        .map(|backup| backup.unwrap())
        .filter(|backup| {
            // Ignora arquivos que não são backups
            NaiveDate::parse_from_str(backup.file_name().to_str().unwrap(), "%Y%m%d").is_ok()
        })
        .collect();

        let num_backups = backup_list.len();
        let backups_to_remove: Vec<DirEntry> = backup_list
            .into_iter()
            .filter(|backup| {
                let today = Utc::today().naive_utc();
                // NOTE: seguro, já que testamos que podemos fazer a conversão no
                // filter de backup_list
                let backup_date =
                    NaiveDate::parse_from_str(backup.file_name().to_str().unwrap(), "%Y%m%d")
                        .unwrap();
                (today - backup_date).num_days() >= config.remove_older_than
            })
            .collect();

        // Impede (por segurança) que o programa remova todos os backups
        if num_backups - backups_to_remove.len() >= 1 {
            let mut handles = vec![];
            for backup in backups_to_remove {
                handles.push(thread::spawn(move || {
                    println!("Removing backup '{}'", backup.file_name().to_str().unwrap());
                    fs::remove_dir_all(backup.path()).unwrap();
                }));
            }
            for handle in handles {
                handle.join().unwrap();
            }
        }

        Ok(())
    }
}

// #######################
//         Erros
// #######################
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
