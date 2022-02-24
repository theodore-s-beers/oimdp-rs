use anyhow::Result;
use std::{env, fs};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut file_path = "test.md";
    if args.len() > 1 {
        file_path = &args[1];
    }

    let full_text = fs::read_to_string(file_path)?;
    let text_parsed = oimdp_rs::parser(full_text).unwrap();

    println!("Parsed {} content items", text_parsed.content.len());

    Ok(())
}
