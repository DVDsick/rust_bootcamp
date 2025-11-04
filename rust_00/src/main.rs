fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Show help if requested
    if args.contains(&"--help".to_string()) || args.contains(&"-h".to_string()) {
        println!(
            "Usage: hello [OPTIONS] [NAME]\n\
            \nArguments:\n\
            [NAME] Name to greet [default: World]\n\
            \nOptions:\n\
            --upper Convert to uppercase\n\
            --repeat Repeat greeting N times [default: 1]\n\
            -h, --help Print help"
        );
        return;
    }

    // Defaults
    let mut name = "World".to_string();
    let mut repeat = 1;
    let mut upper = false;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--upper" => {
                upper = true;
                if i + 1 < args.len() {
                    name = args[i + 1].clone();
                    i += 1;
                }
            }
            "--repeat" => {
                if i + 2 < args.len() {
                    repeat = args[i + 1].parse().unwrap_or(1);
                    name = args[i + 2].clone();
                    i += 2;
                }
            }
            n if !n.starts_with("--") && !n.starts_with("-") => {
                name = n.to_string();
            }
            _ => {}
        }
        i += 1;
    }

    for _ in 0..repeat {
        let output = format!("Hello, {}!", name);
        if upper {
            println!("{}", output.to_uppercase());
        } else {
            println!("{}", output);
        }
    }
}
