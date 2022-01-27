use crate::{
    config::Config,
    error::{BackyError, BackyResult},
};
use chrono::{NaiveDate, Utc};
use std::process::{self, Stdio};
use std::{
    fs,
    os::unix::fs::symlink,
    path::{Path, PathBuf},
    thread,
};
use std::{fs::DirEntry, io};

// #######################
//   Erros
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
    /// Atualiza os arquivos monitorados para a versão mais recente.
    Update,
}

impl Command {
    /// Cria e devolve o comando correspondente à lista argumentos
    pub fn from_args(args: &[String]) -> Result<Self, Box<dyn BackyError>> {
        if args.len() <= 1 {
            return Err(Box::new(ErrNoCommand));
        }
        match args[1].as_str() {
            "help" => Ok(Command::Help),
            "clean" => Ok(Command::Clean),
            "update" => Ok(Command::Update),
            cmd => Err(Box::new(ErrBadCommand {
                cmd: cmd.to_string(),
            })),
        }
    }

    /// Executa este comando
    pub fn execute(self, config: Config) -> BackyResult {
        // TODO: reorganizar essa função quando tiver o restante dos comandos
        match self {
            Command::Clean => clean_archive(config),
            Command::Update => update_archive(config),
            Command::Help => print_help(),
        }
    }
}

// #######################
//   Comandos
// #######################

/// Escreve uma mensagem de ajuda para o usuário.
fn print_help() -> BackyResult {
    println!("{}", HELP_MSG);
    Ok(())
}

/// Cria um novo snapshot do sistema na pasta de archive
fn update_archive(config: Config) -> BackyResult {
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
    let mut handles = vec![];
    for file in config.files {
        let backup_dir = backup_dir.clone();
        let latest_arg = latest_arg.clone();
        handles.push(thread::spawn(move || {
            println!("backing up '{}'...", file);
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
    fs::remove_file(&latest_link).ok();
    if let Err(err) = symlink(&backup_dir, &latest_link) {
        return Err(Box::new(ErrSymCreationFailed { err }));
    };

    Ok(())
}

/// Deleta da memória os arquivos com mais de `config.remove_older_than` dias.
fn clean_archive(config: Config) -> BackyResult {
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
                NaiveDate::parse_from_str(backup.file_name().to_str().unwrap(), "%Y%m%d").unwrap();
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
    return Ok(());
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
fn create_backup_dir(archive_path: &str) -> Result<PathBuf, Box<dyn BackyError>> {
    let today = Utc::today().format("%Y%m%d/").to_string();
    let mut backup_dir = PathBuf::from(archive_path);
    backup_dir.push(&today);
    match fs::create_dir_all(&backup_dir) {
        Err(err) => Err(Box::new(ErrSymCreationFailed { err })),
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
