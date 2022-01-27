use crate::error::BackyResult;
use super::BackyCommand;

const HELP_MSG: &'static str = "\
Backy helps users to manage local and remote backups using the rclone and rsync tools.

USAGE:
    backy [COMMAND]

where COMMAND is one of:
    help      Write this help message.
    update    Update backup files to most recent version.
    clean     Remove old backups.";

/// Escreve a mensagem de ajuda para o usuário.
pub struct CmdHelp;
impl BackyCommand for CmdHelp {
    fn execute(&self, _config: crate::config::Config) -> BackyResult {
        println!("{}", HELP_MSG);
        Ok(())
    }
}
