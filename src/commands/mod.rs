mod utils;

/// Comandos (ou modos de operação) que o programa pode ser executado.
pub enum Command {
    /// Remove backups antigos.
    Clean,
    /// Escreve a mensagem de ajuda.
    Help,
    /// Atualiza os arquivos monitorados para a versão mais recenete.
    Update,
    /// Acaba a execução do programa com uma mensagem de erro.
    Exit(&'static str),
}

impl Command {
    /// Cria e devolve o comando correspondente à lista argumentos
    pub fn from_args(args: &[String]) -> Self {
        if args.len() <= 1 {
            return Command::Exit("Número insuficiente de argumentos");
        }
        return match args[1].as_str() {
            "help" => Command::Help,
            "clean" => Command::Clean,
            "update" => Command::Update,
            _ => Command::Exit("Este comando não existe"),
        };
    }

    /// Executa este comando
    pub fn execute(self) {
        match self {
            Command::Clean => println!("Chamando clean!"),
            Command::Update => println!("Chamando update!"),
            Command::Help => utils::print_help(),
            Command::Exit(msg) => utils::print_error(msg),
        }
    }
}
