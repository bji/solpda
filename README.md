## solpda -- A utility for generating Solana Program Derived Addresses ##

# Introduction #

solpda is a small utility program that calculates a Program Derived Address given a program address and seed bytes.
This can be useful to compute PDAs which are inputs to other processes, such as compiled into a program.

Using solpda is simple.  You supply the program address and seed bytes on the command line, and solpda finds the
PDA and prints it.

solpda actually has two modes: PDA and "pubkey".  The pubkey mode is just a convenience that allows a pubkey to
be displayed in several forms.  It is most useful for converting a Base58 encoded address into an array of bytes;
or for converting an array of bytes into a Base58 encoded address.

```
$ solpda --help

Usage: solpda [--help]
       solpda [--no-bump-seed] [--bytes] <PROGRAM_ID> <SEED>...
       solpda -pubkey [--bytes] <PROGRAM_ID>

  solpda computes the Solana Program Derived Address for a given program and
  set of seeds.  It outputs the PDA as either an array of byte values if the
  --bytes option is provided, or as a Base58-encoded address if not.  Unless
  [--no-bump-seeed] is specified, it also appends a bump seed automatically
  starting with 255 and reducing down to 0 until a valid PDA is found, and
  also outputs the "bump seed" that was used to derive the PDA.

  <PROGRAM_ID> is either the Base58-encoded address of the program for
    which to compute the PDA, or a file containing a JSON array of the bytes
    of the same, or a array of u8 bytes.

  One or more <SEED> values are provided.  Each SEED is specified as:

    u8[values]     : values is a comma-separated list of numbers in the
                     range [0, 255]
    u16[values]    : values is a comma-separated list of numbers in the
                     range [0, 65535]
    u32[values]    : values is a comma-separated list of numbers in the
                     range [0, 4294967295]
    u64[values]    : values is a comma-separated list of numbers in the
                     range [0, 18446744073709551615]
    String[value]  : value is a string
    Pubkey[value] : value is a Base58-encoded ed25519 public key
    Sha256[SEED]   : value is a SEED (i.e. u8(10))

  If [--bytes] was specified, then the PDA is output as a byte array, else the
  PDA is output as a Base58-encoded string.

  Unless [--no-bump-seed] was specified, the PDA is first output and then the
  seed is output as ".SEED"

  Example:
    $ PROGRAM_ID=TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA

    $ solpda --no-bump-seed $PROGRAM_ID u8[5, 6] String[Hello, world!]
          bad one

    $ solpda $PROGRAM_ID u8[5, 6] String[Hello, world!] u8[10]
      xxxxxxx

    $ solpda --bytes $PROGRAM_ID u8[5, 6] String[Hello, world!]
      xxxxxxx.yyy

    $ solpda --bytes $PROGRAM_ID u8[5, 6] String[Hello, world!]
      [1, 2, 3].yyy

  As a convenience, solpda also supports the -pubkey argument which causes
  it to do nothing other than read the <PROGRAM_ID> argument, which is
  either a Base58-encoded public key, or a key file, or an array of u8
  bytes, and print out the public key that was read in, as either an array
  of bytes (if --bytes was specified), or as a Base58-encoded string (if
  --bytes was not specified).
```
