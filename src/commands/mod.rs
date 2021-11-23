use crate::{config::Config, error::BackyError};
use chrono::Utc;
use std::process::{self, Stdio};
use std::{
    fs,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
};

const HELP_MSG: &'static str = "\
Backy helps users to manage local and remote backups using the rclone and rsync tools.

USAGE:
    backy [COMMAND]

where COMMAND is one of:
    help      Write this help message.
    update    Update backup files to most recent version.
    clean     Remove old backups.";

/// Comandos (ou modos de operação) que o programa pode ser executado.
pub enum Command {
    /// Remove backups antigos.
    Clean,
    /// Escreve a mensagem de ajuda.
    Help,
    /// Atualiza os arquivos monitorados para a versão mais recenete.
    Update,
}

impl Command {
    /// Cria e devolve o comando correspondente à lista argumentos
    pub fn from_args(args: &[String]) -> Result<Self, BackyError> {
        if args.len() <= 1 {
            return Err(BackyError::NoCommand);
        }
        match args[1].as_str() {
            "help" => Ok(Command::Help),
            "clean" => Ok(Command::Clean),
            "update" => Ok(Command::Update),
            cmd => Err(BackyError::BadCommand(cmd.to_string())),
        }
    }

    /// Executa este comando
    pub fn execute(self, config: Config) -> Result<(), BackyError> {
        // TODO: reorganizar essa função quando tiver o restante dos comandos
        match self {
            Command::Clean => Ok(()),
            Command::Update => update_archive(config),
            Command::Help => print_help(),
        }
    }
}

// #######################
//   Comandos
// #######################

/// Escreve uma mensagem de ajuda para o usuário.
fn print_help() -> Result<(), BackyError> {
    println!("{}", HELP_MSG);
    Ok(())
}

/// Cria um novo snapshot do sistema na pasta de archive
fn update_archive(config: Config) -> Result<(), BackyError> {
    if !user_has_rsync() {
        return Err(BackyError::NoRsync);
    }

    // Checa se todos os arquivos que devem ser armazenados existem
    let inexistent_files = inexistent_files(&config.files);
    if inexistent_files.len() > 0 {
        return Err(BackyError::BadFiles(inexistent_files));
    }

    // Cria o diretório do backup de hoje
    let backup_dir = create_backup_dir(&config.archive_path)?;
    let latest_link = format!("{}/latest", &config.archive_path);
    let latest_arg = format!("--link-dest={}", latest_link);

    // Faz o backup de cada um dos arquivos
    for file in &config.files {
        println!("backing up '{}'...", file);
        let _rsync_status = process::Command::new("rsync")
            .current_dir(&backup_dir)
            .args(["-az", "--delete", file, &latest_arg, "."])
            .status()
            .unwrap();
    }

    // Recria o link simbólico para latest
    fs::remove_file(&latest_link).ok();
    if let Err(err) = symlink(&backup_dir, &latest_link) {
        return Err(BackyError::SymCreationFailed(err));
    };

    Ok(())
}

// #######################
//   Definições auxiliares
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
fn create_backup_dir(archive_path: &str) -> Result<PathBuf, BackyError> {
    let today = Utc::today().format("%Y%m%d/").to_string();
    let mut backup_dir = PathBuf::from(archive_path);
    backup_dir.push(&today);
    match fs::create_dir_all(&backup_dir) {
        Err(err) => Err(BackyError::NoArchiveDir(err)),
        _ => Ok(backup_dir),
    }
}

/// Checa se o usuário tem o programa `rsync` instalado
fn user_has_rsync() -> bool {
    process::Command::new("rsync")
        .arg("--version")
        .stdout(Stdio::null())
        .status()
        .is_ok()
}
