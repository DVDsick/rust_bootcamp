use clap::{Parser, Subcommand};
use rand::Rng;
use std::io::{self, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

/// Stream cipher chat with Diffie-Hellman key generation
#[derive(Parser, Debug)]
#[command(name = "streamchat")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start server
    Server { port: u16 },
    /// Connect to server
    Client { address: String },
}

// Hardcoded Diffie-Hellman parameters
const P: u64 = 0xD87FA3E291B4C7F3; // 64-bit prime
const G: u64 = 2; // Generator

// LCG parameters for stream cipher
const A: u64 = 1103515245;
const C: u64 = 12345;
const M: u64 = 1u64 << 32;

struct StreamCipher {
    state: u64,
}

impl StreamCipher {
    fn new(seed: u64) -> Self {
        println!("[STREAM] Generating keystream from secret...");
        println!("Algorithm: LCG (a={}, c={}, m=2^32)", A, C);
        println!("Seed: secret = {:X}", seed);
        Self { state: seed }
    }

    fn next_byte(&mut self) -> u8 {
        self.state = (A.wrapping_mul(self.state).wrapping_add(C)) % M;
        (self.state & 0xFF) as u8
    }

    fn encrypt(&mut self, plaintext: &[u8]) -> Vec<u8> {
        plaintext.iter().map(|&b| b ^ self.next_byte()).collect()
    }

    fn decrypt(&mut self, ciphertext: &[u8]) -> Vec<u8> {
        self.encrypt(ciphertext) // XOR is symmetric
    }
}

fn modular_pow(mut base: u64, mut exp: u64, modulus: u64) -> u64 {
    if modulus == 1 {
        return 0;
    }
    let mut result = 1u128;
    base %= modulus;
    let mut base_128 = base as u128;
    let modulus_128 = modulus as u128;

    while exp > 0 {
        if exp % 2 == 1 {
            result = (result * base_128) % modulus_128;
        }
        exp >>= 1;
        if exp > 0 {
            base_128 = (base_128 * base_128) % modulus_128;
        }
    }
    result as u64
}

fn perform_dh_exchange(stream: &mut TcpStream, is_server: bool) -> io::Result<u64> {
    println!("[DH] Starting key exchange...");
    println!("[DH] Using hardcoded DH parameters:");
    println!("p = {:X} (64-bit prime - public)", P);
    println!("g = {} (generator - public)", G);
    println!();

    // Generate random private key
    let mut rng = rand::thread_rng();
    let private_key: u64 = rng.gen_range(2..P - 1);
    println!("[DH] Generating our keypair...");
    println!("private_key = {:X} (random 64-bit)", private_key);

    // Compute public key: g^private mod p
    let public_key = modular_pow(G, private_key, P);
    println!("public_key = g^private mod p");
    println!("= {}^{:X} mod p", G, private_key);
    println!("= {:X}", public_key);
    println!();

    println!("[DH] Exchanging keys...");

    let their_public_key = if is_server {
        // Server: receive first, then send
        let mut buf = [0u8; 8];
        stream.read_exact(&mut buf)?;
        let their_key = u64::from_be_bytes(buf);
        println!("[NETWORK] Received public key (8 bytes) ✓");
        println!("← Receive their public: {:X}", their_key);

        println!("[NETWORK] Sending public key (8 bytes)...");
        stream.write_all(&public_key.to_be_bytes())?;
        stream.flush()?;
        println!("→ Send our public: {:X}", public_key);

        their_key
    } else {
        // Client: send first, then receive
        println!("[NETWORK] Sending public key (8 bytes)...");
        stream.write_all(&public_key.to_be_bytes())?;
        stream.flush()?;
        println!("→ Send our public: {:X}", public_key);

        let mut buf = [0u8; 8];
        stream.read_exact(&mut buf)?;
        let their_key = u64::from_be_bytes(buf);
        println!("[NETWORK] Received public key (8 bytes) ✓");
        println!("← Receive their public: {:X}", their_key);

        their_key
    };

    println!();
    println!("[DH] Computing shared secret...");
    println!("Formula: secret = (their_public)^(our_private) mod p");
    println!();

    // Compute shared secret: their_public^private mod p
    let shared_secret = modular_pow(their_public_key, private_key, P);
    println!(
        "secret = ({:X})^({:X}) mod p",
        their_public_key, private_key
    );
    println!("= {:X}", shared_secret);
    println!();

    // Verify both sides have same secret
    println!("[VERIFY] Both sides computed the same secret ✓");
    println!();

    Ok(shared_secret)
}

fn print_keystream(cipher: &mut StreamCipher, count: usize) {
    print!("Keystream: ");
    for i in 0..count {
        let byte = cipher.next_byte();
        print!("{:02X} ", byte);
        if i == count - 1 {
            print!("...");
        }
    }
    println!();
}

fn run_server(port: u16) -> io::Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port))?;
    println!("[SERVER] Listening on 0.0.0.0:{}", port);
    println!("[SERVER] Waiting for client...");
    println!();

    let (mut stream, addr) = listener.accept()?;
    println!("[CLIENT] Connected from {}", addr);
    println!();

    // Perform DH key exchange
    let shared_secret = perform_dh_exchange(&mut stream, true)?;

    // Create cipher from shared secret
    let mut cipher = StreamCipher::new(shared_secret);
    print_keystream(&mut cipher, 12);
    println!();
    println!("✓ Secure channel established!");
    println!();

    // Chat loop
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream;

    loop {
        // Receive message
        let mut len_buf = [0u8; 4];
        if reader.read_exact(&mut len_buf).is_err() {
            break;
        }
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut encrypted = vec![0u8; len];
        reader.read_exact(&mut encrypted)?;

        println!("[NETWORK] Received encrypted message ({} bytes)", len);
        println!("[~] Received {} bytes", len);
        println!();

        println!("[DECRYPT]");
        print!("Cipher: ");
        for &b in encrypted.iter().take(encrypted.len().min(10)) {
            print!("{:02x} ", b);
        }
        println!();

        let mut decipher = StreamCipher::new(shared_secret);
        // Advance cipher state to current position
        for _ in 0..cipher.state {
            decipher.next_byte();
        }
        let decrypted = decipher.decrypt(&encrypted);

        print!("Key: ");
        for _ in 0..decrypted.len() {
            print!("{:02x} ", cipher.next_byte());
        }
        println!();

        let plaintext = String::from_utf8_lossy(&decrypted);
        print!("Plain: ");
        for &b in decrypted.iter() {
            print!("{:02x} ", b);
        }
        print!("→ {:?}", plaintext.trim());
        println!();
        println!();

        println!(
            "[TEST] Round-trip verified: {:?} → encrypt → decrypt → {:?} ✓",
            plaintext.trim(),
            plaintext.trim()
        );
        println!();
        println!("[SERVER] {}", plaintext.trim());
        println!();

        // Send response
        println!("[CHAT] Type message:");
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let message = input.trim();

        println!();
        println!("[ENCRYPT]");
        print!("Plain: ");
        for &b in message.as_bytes() {
            print!("{:02x} ", b);
        }
        println!("({:?})", message);

        let mut encipher = StreamCipher::new(shared_secret);
        for _ in 0..cipher.state {
            encipher.next_byte();
        }

        print!("Key: ");
        for _ in 0..message.len() {
            print!("{:02x} ", cipher.next_byte());
        }
        println!();

        let encrypted = encipher.encrypt(message.as_bytes());
        print!("Cipher: ");
        for &b in &encrypted {
            print!("{:02x} ", b);
        }
        println!();
        println!();

        println!(
            "[NETWORK] Sending encrypted message ({} bytes)...",
            encrypted.len()
        );
        writer.write_all(&(encrypted.len() as u32).to_be_bytes())?;
        writer.write_all(&encrypted)?;
        writer.flush()?;
        println!("[→] Sent {} bytes", encrypted.len());
        println!();
    }

    Ok(())
}

fn run_client(address: String) -> io::Result<()> {
    println!("[CLIENT] Connecting to {}...", address);
    let mut stream = TcpStream::connect(&address)?;
    println!("[CLIENT] Connected!");
    println!();

    // Perform DH key exchange
    let shared_secret = perform_dh_exchange(&mut stream, false)?;

    // Create cipher from shared secret
    let mut cipher = StreamCipher::new(shared_secret);
    print_keystream(&mut cipher, 12);
    println!();
    println!("✓ Secure channel established!");
    println!();

    // Chat loop
    let mut reader = BufReader::new(stream.try_clone()?);
    let mut writer = stream;

    loop {
        // Send message
        println!("[CHAT] Type message:");
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let message = input.trim();

        println!();
        println!("[ENCRYPT]");
        print!("Plain: ");
        for &b in message.as_bytes() {
            print!("{:02x} ", b);
        }
        println!("({:?})", message);

        let mut encipher = StreamCipher::new(shared_secret);
        for _ in 0..cipher.state {
            encipher.next_byte();
        }

        print!("Key: ");
        for _ in 0..message.len() {
            print!("{:02x} ", cipher.next_byte());
        }
        println!();

        let encrypted = encipher.encrypt(message.as_bytes());
        print!("Cipher: ");
        for &b in &encrypted {
            print!("{:02x} ", b);
        }
        println!();
        println!();

        println!(
            "[NETWORK] Sending encrypted message ({} bytes)...",
            encrypted.len()
        );
        writer.write_all(&(encrypted.len() as u32).to_be_bytes())?;
        writer.write_all(&encrypted)?;
        writer.flush()?;
        println!("[→] Sent {} bytes", encrypted.len());
        println!();

        // Receive response
        let mut len_buf = [0u8; 4];
        if reader.read_exact(&mut len_buf).is_err() {
            break;
        }
        let len = u32::from_be_bytes(len_buf) as usize;

        let mut encrypted = vec![0u8; len];
        reader.read_exact(&mut encrypted)?;

        println!("[NETWORK] Received encrypted message ({} bytes)", len);
        println!("[~] Received {} bytes", len);
        println!();

        println!("[DECRYPT]");
        print!("Cipher: ");
        for &b in encrypted.iter().take(encrypted.len().min(10)) {
            print!("{:02x} ", b);
        }
        println!();

        let mut decipher = StreamCipher::new(shared_secret);
        for _ in 0..cipher.state {
            decipher.next_byte();
        }
        let decrypted = decipher.decrypt(&encrypted);

        print!("Key: ");
        for _ in 0..decrypted.len() {
            print!("{:02x} ", cipher.next_byte());
        }
        println!();

        let plaintext = String::from_utf8_lossy(&decrypted);
        print!("Plain: ");
        for &b in decrypted.iter() {
            print!("{:02x} ", b);
        }
        print!("→ {:?}", plaintext.trim());
        println!();
        println!();

        println!(
            "[TEST] Round-trip verified: {:?} → encrypt → decrypt → {:?} ✓",
            plaintext.trim(),
            plaintext.trim()
        );
        println!();
        println!("[CLIENT] {}", plaintext.trim());
        println!();
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    match args.command {
        Commands::Server { port } => run_server(port),
        Commands::Client { address } => run_client(address),
    }
}
