# Backy

Esse projeto surgiu da necessidade pessoal de melhorar o meu _workflow_ de backups automatizados usando as ferramentas [rclone](rclone.org) e [rsync](rsync.samba.org). Os principais problemas que eu encontrei em minha implementação em bash eram:

- Tratamento pouco robusto de erros, que levava a comportamentos inesperados e potencial perda de dados. 

- Unificação de código e configuração, o que poluía o histórico do git e também exigia mais esforço mental para mudar algo simples como quais os arquivos que deveriam estar no backup. 

Esses e muitos outros problemas me fizeram repensar na estratégia como um todo e, finalmente, decidi reescrever o projeto em [Rust](rust-lang.org).

