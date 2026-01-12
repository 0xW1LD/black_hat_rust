use sha1::Digest;
use std::{
    env,
    error::Error,
    fs::File,
    io::{BufRead,BufReader},
};

const SHA1_HEX_STRING_LENGTH: usize = 40;

fn main() -> Result<(), Box<dyn Error>>{
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        println!("Usage:");
        println!("{} <wordlist.txt> <sha1_hash>",args[0]);
        return Ok(());
    }

    let hash = args[2].trim();
    if hash.len() != SHA1_HEX_STRING_LENGTH {
        return Err("Sha1 hash is not valid".into());
    }

    let wordlist: File = File::open(&args[1])?;
    let reader: BufReader<&File> = BufReader::new(&wordlist);

    for line in reader.lines() {
        let line: String = line?;
        let password: &str = line.trim();

        if hash == &hex::encode(sha1::Sha1::digest(password.as_bytes())){
            println!("Password found: {}",&password);
            return Ok(());
        }
    }

    println!("Password not found in wordlist.");

    Ok(())
}
