use crate::error::BackyError;

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
    pub fn execute(self) {
        match self {
            Command::Clean => println!("Chamando clean!"),
            Command::Update => println!("Chamando update!"),
            Command::Help => print_help(),
        }
    }
}

const HELP_MSG: &'static str = "\
Backy auxilia na manutenção de backups locais e remotos usando as ferramentas rclone e rsync.

USO:
    backy [COMANDO]

Onde COMANDO pode ser:
    help      Escreve essa mensagem de ajuda
    update    Atualiza os arquivos monitorados para versão mais recente.
    clean     Remove os backups antigos.";

pub fn print_help() {
    println!("{}", HELP_MSG);
}
