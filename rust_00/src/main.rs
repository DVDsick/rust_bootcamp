use std::env;

/// Print usage/help information
fn print_help() {
    println!("Rusty Hello - CLI arguments et ownership\n");
    println!("Usage: rust_00 [OPTIONS] [NAME]\n");
    println!("Arguments:");
    println!("  [NAME]              Name to greet [default: World]");
    println!("\nOptions:");
    println!("      --upper         Convert to uppercase");
    println!("      --repeat N      Repeat greeting N times [default: 1]");
    println!("  -h, --help         Print help");
}

fn main() {
    let mut name: Option<String> = None;
    let mut upper = false;
    let mut repeat: usize = 1;

    let mut args = env::args().skip(1).peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                return;
            }
            "--upper" => upper = true,
            "--repeat" => {
                if let Some(n) = args.next() {
                    repeat = n.parse().unwrap_or(1);
                } else {
                    eprintln!("--repeat requires a number");
                    return;
                }
            }
            _ => {
                // First positional argument is name
                if name.is_none() {
                    name = Some(arg);
                } else {
                    // Extra positional args are ignored
                }
            }
        }
    }

    let name = name.as_deref().unwrap_or("World");
    let greeting = format!("Hello, {}!", name);

    for _ in 0..repeat {
        if upper {
            println!("{}", greeting.to_uppercase());
        } else {
            println!("{}", greeting);
        }
    }
}
