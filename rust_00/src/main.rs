use clap::Parser;

/// Rusty Hello - CLI arguments et ownership
#[derive(Parser, Debug)]
#[command(name = "hello")]
struct Args {
    /// Name to greet [default: World]
    name: Option<String>,

    /// Convert to uppercase
    #[arg(long)]
    upper: bool,

    /// Repeat greeting N times [default: 1]
    #[arg(long, default_value_t = 1)]
    repeat: usize,
}

fn main() {
    let args = Args::parse();

    let name = args.name.as_deref().unwrap_or("World");
    let greeting = format!("Hello, {}!", name);

    for _ in 0..args.repeat {
        if args.upper {
            println!("{}", greeting.to_uppercase());
        } else {
            println!("{}", greeting);
        }
    }
}
