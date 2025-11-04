use clap::Parser;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};

/// Hex Tool - Read & Write Binary Files
#[derive(Parser, Debug)]
#[command(name = "hextool")]
#[command(about = "Read and write binary files in hexadecimal")]
struct Args {
    /// Target file
    #[arg(short, long)]
    file: String,

    /// Read mode (display hex)
    #[arg(short, long)]
    read: bool,

    /// Write mode (hex string to write)
    #[arg(short, long)]
    write: Option<String>,

    /// Offset in bytes (decimal or 0x hex)
    #[arg(short, long, default_value = "0", value_parser = parse_offset)]
    offset: u64,

    /// Number of bytes to read
    #[arg(short, long)]
    size: Option<usize>,
}

fn parse_offset(s: &str) -> Result<u64, String> {
    if s.starts_with("0x") || s.starts_with("0X") {
        u64::from_str_radix(&s[2..], 16)
            .map_err(|_| format!("Invalid hex offset: {}", s))
    } else {
        s.parse::<u64>()
            .map_err(|_| format!("Invalid decimal offset: {}", s))
    }
}

fn hex_to_bytes(hex: &str) -> Option<Vec<u8>> {
    let hex = hex.trim();
    if hex.len() % 2 != 0 {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

fn byte_to_ascii(byte: u8) -> char {
    if byte >= 0x20 && byte <= 0x7E {
        byte as char
    } else {
        '.'
    }
}

fn main() -> io::Result<()> {
    let args = Args::parse();

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
            .open(&args.file)?;
        file.seek(SeekFrom::Start(args.offset))?;
        file.write_all(&data)?;
        
        println!("Writing {} bytes at offset 0x{:08x}", data.len(), args.offset);
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
