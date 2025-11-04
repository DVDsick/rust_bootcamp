use clap::Parser;

/// Rusty Hello - CLI arguments et ownership
#[derive(Parser, Debug)]
#[command(name = "hello")]
struct Args {
    /// Convert to uppercase
    #[arg(long)]
    upper: bool,

    /// Repeat greeting N times [default: 1]
    #[arg(long, default_value_t = 1)]
    repeat: usize,

    /// Name to greet [default: World]
    name: Option<String>,
}

fn main() {
    let args = Args::parse();

    // Use &str where possible; fall back to "World"
    let name: &str = args.name.as_deref().unwrap_or("World");

    let mut greeting = format!("Hello, {}!", name);

    if args.upper {
        greeting = greeting.to_uppercase();
    }

    for _ in 0..args.repeat {
        println!("{}", greeting);
    }
}
