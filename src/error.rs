use std::io;

/// Um erro encontrado durante a execução do programa.
pub trait BackyError {
    fn display(&self) {
        let msg = self.get_err_msg();
        eprintln!("Error: {}", msg);
    }
    fn get_err_msg(&self) -> &str;
}

/// Não foi passado um comando par o programa
struct ErrNoCommand;
impl BackyError for ErrNoCommand {
    fn get_err_msg(&self) -> &str {
        "no command to execute. Try `backy help` for aditional information."
    }
}

/// O comando passado não existe
struct ErrBadCommand {
    cmd: String,
}
impl BackyError for ErrBadCommand {
    fn get_err_msg(&self) -> &str {
        format!("command '{}' doesn't exist", self.cmd).as_str()
    }
}

/// Não foi possível abrir o arquivo de configuração
struct ErrNoConfigFile;
impl BackyError for ErrNoConfigFile {
    fn get_err_msg(&self) -> &str {
        "unable to open the configuration file (maybe the file doesn't exist?)"
    }
}

/// Não foi possível encontrar o diretório de configuração
struct ErrNoConfigDir;
impl BackyError for ErrNoConfigDir {
    fn get_err_msg(&self) -> &str {
        "unable to open the configuration directory"
    }
}

/// O arquivo de configuração contém erros de formatação ou typos
struct ErrBadConfigFormat {
    err: toml::de::Error,
}
impl BackyError for ErrBadConfigFormat {
    fn get_err_msg(&self) -> &str {
        format!("unable to parse config:\n{}", self.err).as_str()
    }
}

/// Não foi possível criar o diretório de backup
struct ErrNoArchiveDir {
    err: io::Error,
}
impl BackyError for ErrNoArchiveDir {
    fn get_err_msg(&self) -> &str {
        format!("unable to create backup dir:\n{}", self.err).as_str()
    }
}

/// Alguns dos arquivos de backup não existem no sistema
struct ErrBadFiles {
    missing_files: Vec<String>,
}
impl BackyError for ErrBadFiles {
    fn get_err_msg(&self) -> &str {
        let mut msg = String::from("unable to find the following files:");
        for file in self.missing_files {
            msg.push('\n');
            msg.push_str(&file);
        }
        msg.as_str()
    }
}

/// Não foi possível criar o link simbólico nos archives
struct ErrSymCreationFailed {
    err: io::Error,
}
impl BackyError for ErrSymCreationFailed {
    fn get_err_msg(&self) -> &str {
        format!("unable to create `latest` symlink:\n{}", self.err).as_str()
    }
}

/// Não foi possível encontrar o comando rsync no PATH do usuário
struct ErrNoRsync;
impl BackyError for ErrNoRsync {
    fn get_err_msg(&self) -> &str {
        "unable to find `rsync` executable"
    }
}
