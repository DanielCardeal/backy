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
