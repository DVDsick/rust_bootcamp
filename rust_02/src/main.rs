use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};

use std::env;

/// Hex Tool - Read & Write Binary Files
struct Args {
    file: String,
    read: bool,
    write: Option<String>,
    offset: u64,
    size: Option<usize>,
}

fn print_help() {
    println!("Hex Tool - Read & Write Binary Files\n");
    println!("Usage: hextool --file <PATH> [--read | --write <HEX>] [--offset <N>] [--size <N>]\n");
    println!(
        "Options:\n  -f, --file PATH      Target file (required)\n      --read           Read mode (display hex)\n      --write HEX      Write mode (hex string to write)\n      --offset N       Offset in bytes (decimal or 0x hex) [default: 0]\n      --size N         Number of bytes to read\n  -h, --help           Print help"
    );
}

fn parse_args() -> Result<Args, String> {
    let mut file: Option<String> = None;
    let mut read = false;
    let mut write: Option<String> = None;
    let mut offset: u64 = 0;
    let mut size: Option<usize> = None;

    let mut it = env::args().skip(1).peekable();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                std::process::exit(0);
            }
            "-f" | "--file" => {
                file = it.next();
            }
            "-r" | "--read" => read = true,
            "-w" | "--write" => write = it.next(),
            "-o" | "--offset" => {
                if let Some(v) = it.next() {
                    offset = parse_offset(&v)?;
                }
            }
            "-s" | "--size" => {
                if let Some(v) = it.next() {
                    size = Some(v.parse().unwrap_or(16));
                }
            }
            _ => {
                eprintln!("error");
                std::process::exit(2);
            }
        }
    }

    let file = file.ok_or_else(|| "--file is required".to_string())?;
    Ok(Args {
        file,
        read,
        write,
        offset,
        size,
    })
}

fn parse_offset(s: &str) -> Result<u64, String> {
    if s.starts_with("0x") || s.starts_with("0X") {
        u64::from_str_radix(&s[2..], 16).map_err(|_| format!("Invalid hex offset: {}", s))
    } else {
        s.parse::<u64>()
            .map_err(|_| format!("Invalid decimal offset: {}", s))
    }
}

fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
    let hex = hex.trim();
    if !hex.len().is_multiple_of(2) {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

fn byte_to_ascii(byte: u8) -> char {
    if (0x20..=0x7E).contains(&byte) {
        byte as char
    } else {
        '.'
    }
}

fn main() -> io::Result<()> {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            print_help();
            std::process::exit(2);
        }
    };

    // Write Mode
    if let Some(hexstr) = &args.write {
        let bytes = hex_to_bytes(hexstr);
        if bytes.is_none() {
            eprintln!("Invalid hex string.");
            std::process::exit(1);
        }
        let data = bytes.unwrap();
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(&args.file)?;
        file.seek(SeekFrom::Start(args.offset))?;
        file.write_all(&data)?;

        println!(
            "Writing {} bytes at offset 0x{:08x}",
            data.len(),
            args.offset
        );
        print!("Hex: ");
        for (i, &b) in data.iter().enumerate() {
            if i > 0 {
                print!(" ");
            }
            print!("{:02x}", b);
        }
        println!();
        print!("ASCII: ");
        for &b in &data {
            print!("{}", byte_to_ascii(b));
        }
        println!("\n✓ Successfully written");
        return Ok(());
    }

    // Read Mode
    if args.read {
        let size = args.size.unwrap_or(16);
        let mut file = File::open(&args.file)?;
        file.seek(SeekFrom::Start(args.offset))?;
        let mut buf = vec![0u8; size];
        let n = file.read(&mut buf)?;

        print!("{:08x}: ", args.offset);
        for (i, b) in buf[..n].iter().enumerate() {
            print!("{:02x} ", b);
            if (i + 1) % 8 == 0 && i + 1 != n {
                print!(" ");
            }
        }
        // Fill with spaces if not enough bytes read
        for _ in n..size {
            print!("   ");
        }
        print!("|");
        for &b in &buf[..n] {
            print!("{}", byte_to_ascii(b));
        }
        println!("|");
        return Ok(());
    }

    println!("Please specify either --read or --write option. Use --help for usage.");
    Ok(())
}
