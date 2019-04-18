use dotfiles::cli::Cli;

fn main() {
    let cli = Cli::default();
    if let Err(err) = cli.exec() {
        eprintln!("error: {}", err);
        std::process::exit(1);
    }
}
