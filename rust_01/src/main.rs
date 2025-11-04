use clap::Parser;
use std::collections::HashMap;
use std::io::{self, Read};

/// Count word frequency in text
#[derive(Parser, Debug)]
#[command(name = "wordfreq")]
struct Args {
    /// Text to analyze (or use stdin if not provided)
    text: Vec<String>,

    /// Show top N words
    #[arg(long, default_value_t = 10)]
    top: usize,

    /// Ignore words shorter than N
    #[arg(long = "min-length", default_value_t = 1)]
    min_length: usize,

    /// Case insensitive counting
    #[arg(long)]
    ignore_case: bool,
}

fn main() {
    let args = Args::parse();

    // Get text from args or stdin
    let text = if args.text.is_empty() {
        let mut input = String::new();
        io::stdin().read_to_string(&mut input).unwrap();
        input
    } else {
        args.text.join(" ")
    };

    // Word frequency analysis
    let mut freq: HashMap<String, usize> = HashMap::new();

    for word in text.split_whitespace() {
        let word = word.trim_matches(|c: char| !c.is_alphanumeric());
        if word.len() < args.min_length {
            continue;
        }
        let word = if args.ignore_case {
            word.to_lowercase()
        } else {
            word.to_string()
        };
        *freq.entry(word).or_insert(0) += 1;
    }

    // Sort by frequency
    let mut freq_vec: Vec<_> = freq.into_iter().collect();
    freq_vec.sort_by(|a, b| b.1.cmp(&a.1));

    // Print result
    let top_n = args.top.min(freq_vec.len());

    println!("Word frequency:");
    for (word, count) in freq_vec.iter().take(top_n) {
        println!("{}: {}", word, count);
    }
}
