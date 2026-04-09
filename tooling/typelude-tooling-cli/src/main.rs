use clap::Parser;

fn main() {
    let cli = typelude_tooling_cli::Cli::parse();
    match typelude_tooling_cli::run(cli) {
        Ok(output) => println!("{output}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        },
    }
}
