/**
 * LICENSE: Public Domain
 **/
use sha2::{Digest, Sha256};
use std::str::FromStr;

#[rustfmt::skip]
fn usage_string() -> String
{
    "\nUsage: solpda [--help]\n\
    \x20      solpda [--no-bump-seed] [--bytes] <PROGRAM_ID> <SEED>...\n\
    \x20      solpda -pubkey [--bytes] <PROGRAM_ID>\n\n\
    \x20 solpda computes the Solana Program Derived Address for a given program and\n\
    \x20 set of seeds.  It outputs the PDA as either an array of byte values if the\n\
    \x20 --bytes option is provided, or as a Base58-encoded address if not.  Unless\n\
    \x20 [--no-bump-seeed] is specified, it also appends a bump seed automatically\n\
    \x20 starting with 255 and reducing down to 0 until a valid PDA is found, and\n\
    \x20 also outputs the \"bump seed\" that was used to derive the PDA.\n\n\
    \x20 <PROGRAM_ID> is either the Base58-encoded address of the program for\n\
    \x20   which to compute the PDA, or a file containing a JSON array of the bytes\n\
    \x20   of the same, or a array of u8 bytes.\n\n\
    \x20 One or more <SEED> values are provided.  Each SEED is specified as:\n\n\
    \x20   u8[values]     : values is a comma-separated list of numbers in the\n\
    \x20                    range [0, 255]\n\
    \x20   u16[values]    : values is a comma-separated list of numbers in the\n\
    \x20                    range [0, 65535]\n\
    \x20   u32[values]    : values is a comma-separated list of numbers in the\n\
    \x20                    range [0, 4294967295]\n\
    \x20   u64[values]    : values is a comma-separated list of numbers in the\n\
    \x20                    range [0, 18446744073709551615]\n\
    \x20   String[value]  : value is a string\n\
    \x20   Pubkey[value] : value is a Base58-encoded ed25519 public key\n\
    \x20   Sha256[SEED]   : value is a SEED (i.e. u8(10))\n\n\
    \x20 If [--bytes] was specified, then the PDA is output as a byte array, else the\n\
    \x20 PDA is output as a Base58-encoded string.\n\n\
    \x20 Unless [--no-bump-seed] was specified, the PDA is first output and then the\n\
    \x20 seed is output as \".SEED\"\n\n\
    \x20 Example:\n\
    \x20   $ PROGRAM_ID=TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA\n\n\
    \x20   $ solpda --no-bump-seed $PROGRAM_ID u8[5,6] 'String[Hello, world!]'
    \x20     Cannot find PDA, consider allowing bump seed\n\n\
    \x20   $ solpda $PROGRAM_ID u8[5,6] 'String[Hello, world!]' u8[10]\n\
    \x20     A89GCYdsataUVrFDbrV416NEZnFZoa6X4CR5ZdSPJohC.255\n\n\
    \x20   $ solpda --bytes $PROGRAM_ID u8[5,6] 'String[Hello, world!]'\n\
    \x20     [181,99,247,119,206,49,238,212,128,158,162,102,53,7,236,105,\\\n\
    \x20      123,108,5,22,43,79,12,70,149,227,221,110,66,137,233,124].255\n\n\
    \x20   $ solpda --no-bump-seed --bytes $PROGRAM_ID u8[5,6] 'String[Hello, world!]' u8[10]\n\
    \x20     [42,46,105,65,231,188,62,57,241,154,124,211,106,133,201,219,\\\n\
    \x20      254,69,136,17,107,6,180,194,222,36,56,108,166,70,47,226]\n\n\
    \x20 As a convenience, solpda also supports the -pubkey argument which causes\n\
    \x20 it to do nothing other than read the <PROGRAM_ID> argument, which is\n\
    \x20 either a Base58-encoded public key, or a key file, or an array of u8\n\
    \x20 bytes, and print out the public key that was read in, as either an array\n\
    \x20 of bytes (if --bytes was specified), or as a Base58-encoded string (if\n\
    \x20 --bytes was not specified).\n\n".to_string()
}

struct Pubkey(pub [u8; 32]);

fn u8_list_to_vec(bytes : &str) -> Result<Vec<u8>, String>
{
    bytes
        .replace(" ", "")
        .split(",")
        .map(|s| s.parse::<u8>().map_err(|e| e.to_string()))
        .collect::<Result<Vec<u8>, String>>()
}

const U8_PREFIX : &str = "u8[";
const U16_PREFIX : &str = "u16[";
const U32_PREFIX : &str = "u32[";
const U64_PREFIX : &str = "u64[";
const STRING_PREFIX : &str = "String[";
const PUBKEY_PREFIX : &str = "Pubkey[";
const SHA256_PREFIX : &str = "Sha256[";

fn make_seed(s : &str) -> Vec<u8>
{
    if s.ends_with("]") {
        let s = &s[0..(s.len() - 1)];
        if s.starts_with(U8_PREFIX) {
            return u8_list_to_vec(&s[U8_PREFIX.len()..]).unwrap();
        }
        else if s.starts_with(U16_PREFIX) {
            return s[U16_PREFIX.len()..]
                .replace(" ", "")
                .split(",")
                .map(|s| s.parse::<u16>().unwrap().to_le_bytes())
                .flatten()
                .collect();
        }
        else if s.starts_with(U32_PREFIX) {
            return s[U32_PREFIX.len()..]
                .replace(" ", "")
                .split(",")
                .map(|s| s.parse::<u32>().unwrap().to_le_bytes())
                .flatten()
                .collect();
        }
        else if s.starts_with(U64_PREFIX) {
            return s[U64_PREFIX.len()..]
                .replace(" ", "")
                .split(",")
                .map(|s| s.parse::<u64>().unwrap().to_le_bytes())
                .flatten()
                .collect();
        }
        else if s.starts_with(STRING_PREFIX) {
            return s[STRING_PREFIX.len()..].as_bytes().to_vec();
        }
        else if s.starts_with(PUBKEY_PREFIX) {
            return Pubkey::from_str(&s[PUBKEY_PREFIX.len()..]).unwrap().0.to_vec();
        }
        else if s.starts_with(SHA256_PREFIX) {
            let mut hasher = Sha256::new();
            hasher.update(&make_seed(&s[SHA256_PREFIX.len()..]));
            return hasher.finalize().to_vec();
        }
    }

    eprintln!("Invalid seed: {}", s);
    std::process::exit(-1);
}

fn private_key_bytes_array_to_pubkey(bytes : &str) -> Result<Pubkey, String>
{
    if bytes.starts_with("[") && bytes.ends_with("]") {
        let bytes = &bytes[1..(bytes.len() - 1)];
        Ok(Pubkey(
            ed25519_dalek::Keypair::from_bytes(u8_list_to_vec(&bytes)?.as_slice())
                .map_err(|e| e.to_string())?
                .public
                .to_bytes()
        ))
    }
    else {
        Err("Invalid key file contents".to_string())
    }
}

fn public_key_bytes_array_to_pubkey(bytes : &str) -> Result<Pubkey, String>
{
    if bytes.starts_with("[") && bytes.ends_with("]") {
        let bytes = &bytes[1..(bytes.len() - 1)];
        Ok(Pubkey(
            u8_list_to_vec(&bytes)?.try_into().map_err(|_| "Incorrect number of bytes in public key".to_string())?
        ))
    }
    else {
        Err("Invalid key file contents".to_string())
    }
}

fn bytes_are_curve_point(bytes : &[u8; 32]) -> bool
{
    curve25519_dalek::edwards::CompressedEdwardsY::from_slice(bytes.as_ref()).decompress().is_some()
}

fn try_find_pda(
    pubkey : &Pubkey,
    seed : &[u8],
    bump_seed : Option<u8>
) -> Option<Pubkey>
{
    let mut hasher = Sha256::new();

    hasher.update(&seed);
    if let Some(bump_seed) = bump_seed {
        hasher.update(&[bump_seed]);
    }
    hasher.update(&pubkey.0);
    hasher.update(b"ProgramDerivedAddress");

    let hash = <[u8; 32]>::try_from(hasher.finalize().as_slice()).unwrap();

    if bytes_are_curve_point(&hash) {
        None
    }
    else {
        Some(Pubkey(hash))
    }
}

fn find_pda(
    program_id : &Pubkey,
    seed : &[u8],
    no_bump_seed : bool
) -> Option<(Pubkey, u8)>
{
    if no_bump_seed {
        return try_find_pda(&program_id, seed, None).map(|pk| (pk, 0));
    }
    else {
        // Use the same algorithm as Solana's seed finding algorithm: start the bump seed at 255 and work backwards
        let mut bump_seed = (std::u8::MAX) as i16;

        while bump_seed >= 0 {
            if let Some(pubkey) = try_find_pda(&program_id, seed, Some(bump_seed as u8)) {
                return Some((pubkey, bump_seed as u8));
            }
            bump_seed -= 1;
        }
    }

    None
}

fn print_pubkey_bytes(b : &[u8; 32])
{
    print!("[");
    let mut need_comma = false;
    b.iter().for_each(|b| {
        if need_comma {
            print!(",{}", b);
        }
        else {
            print!("{}", b);
            need_comma = true;
        }
    });
    print!("]");
}

fn main()
{
    let mut no_bump_seed = false;
    let mut bytes = false;
    let mut seeds = Vec::<String>::new();
    let mut pubkey_only = false;

    seeds.extend(std::env::args().skip(1));

    while seeds.len() > 0 {
        match seeds[0].as_str() {
            "--help" => {
                println!("{}", usage_string());
                std::process::exit(0);
            },

            "-pubkey" => {
                pubkey_only = true;
                seeds.remove(0);
            },

            "--no-bump-seed" => {
                no_bump_seed = true;
                seeds.remove(0);
            },

            "--bytes" => {
                bytes = true;
                seeds.remove(0);
            },

            _ => break
        }
    }

    if seeds.len() < 1 {
        eprintln!("{}", usage_string());
        std::process::exit(-1);
    }

    let program_id = seeds.remove(0);
    let program_id : Pubkey = std::fs::read_to_string(&program_id)
        .map_err(|e| e.to_string())
        .and_then(|pk_bytes| private_key_bytes_array_to_pubkey(&pk_bytes))
        .or_else(|_| Pubkey::from_str(&program_id))
        .or_else(|_| public_key_bytes_array_to_pubkey(&program_id))
        .unwrap_or_else(|e| {
            eprintln!("Invalid program id: {}", e);
            std::process::exit(-1);
        });

    if pubkey_only {
        if bytes {
            print_pubkey_bytes(&program_id.0);
            println!("");
        }
        else {
            println!("{}", program_id);
        }
        return;
    }

    if seeds.len() < 1 {
        eprintln!("{}", usage_string());
        std::process::exit(-1);
    }

    let seeds : Vec<u8> = seeds.iter().map(|seed| make_seed(seed)).flatten().collect();

    if let Some((pda, bump_seed)) = find_pda(&program_id, seeds.as_slice(), no_bump_seed) {
        if no_bump_seed {
            if bytes {
                print_pubkey_bytes(&pda.0);
                println!("");
            }
            else {
                println!("{}", pda);
            }
        }
        else if bytes {
            print_pubkey_bytes(&pda.0);
            println!(".{}", bump_seed);
        }
        else {
            println!("{}.{}", pda, bump_seed);
        }
    }
    else {
        eprintln!("Cannot find PDA, consider allowing bump seed");
        std::process::exit(1)
    }
}

impl std::str::FromStr for Pubkey
{
    type Err = String;

    fn from_str(s : &str) -> Result<Self, Self::Err>
    {
        let mut address = [0_u8; 32];

        let v = bs58::decode(s).into_vec().map_err(|e| format!("{}", e))?;

        if v.len() == 32 {
            address.copy_from_slice(v.as_slice());
            Ok(Pubkey(address))
        }
        else {
            Err(format!("Invalid address {}", s))
        }
    }
}

impl std::fmt::Display for Pubkey
{
    fn fmt(
        &self,
        f : &mut std::fmt::Formatter
    ) -> std::fmt::Result
    {
        write!(f, "{}", bs58::encode(self.0).into_string())
    }
}
