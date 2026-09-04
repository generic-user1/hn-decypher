# hn-decypher
Decrypts the "dec" file format from the video game [Hacknet](https://store.steampowered.com/app/365450/Hacknet/) without needing any password. In addition to the capability to read a file from your actual filesystem, optionally comes with the capability to read an in-game file from an in-game computer.

## Acknowledgement
The decryption portion of this project is primarily based on relevant code snippits from the game itself. A similar project named [hacknet-decypher](https://github.com/brunnerh/hacknet-decypher/tree/master) did the same thing back in 2020, and although this project isn't based on that one, the two do have a lot of similarities. Additionally, the fact that header values (aside from the "check" string) use a constant passcode *was* directly taken from hacknet-decypher

## Installation
1. Install the [Rust programming language](https://rust-lang.org/tools/install/) if you don't have it already
2. Open a terminal and run the following to install the program: `cargo install --git https://github.com/generic-user1/hn-decypher.git`
    - Save reading functionality is controlled by the `save_read` feature. It is enabled by default, but if you don't want save reading, you can install without save reading using `cargo install --no-default-features --git https://github.com/generic-user1/hn-decypher.git`

## Usage
After installing the program, run `hn-decypher --help` for available options. The exact options available (and the command format) depend on whether you have save reading enabled.

### Examples with save reading enabled
Since it's not possible to copy in-game files from within the game itself, save reading is (intended to be, at least) the most convinient way to use this program with a "dec" file found in the game. The main limitation is that in order for save reading to work, your save file has to exist and be up-to-date with the current state of the game (the in-game save button makes this easy to accomplish).

#### Read and decrypt an in-game file at path "home/Decypher_Test.dec" that is on the player's in-game computer, printing the decrypted output to the terminal:
```
hn-decypher game "home/Decypher_Test.dec"
```
Reading from an in-game file is specified by `game`, and the path of that file is the only required argument. By default, the program assumes you're looking for a file on the player's in-game computer, so specifying a specific in-game computer isn't needed in this case.


#### Read and decrypt a file at path "home/Remote_File.dec" that is on an in-game computer with IP address "1.2.3.4", saving the decrypted output to a file named "decrypted.txt":
```
hn-decypher game --ip-address "1.2.3.4" "home/Remote_File.dec" "decrypted.txt"
```
When two positional arguments are given, the first is interpreted as the target in-game file to read, and the second is interpreted as a real file to write output to. The program supports finding an in-game computer by its IP address (using `--ip-address`), its name (using `--name`), or its internal ID (using `--id`)


#### Read a file at path "home/Copy_Me.dec" that is on the player's in-game computer, and save the file's content directly to a file named "real_file.dec" (without decrypting it)
```
hn-decypher game --verbatim "home/Copy_Me.dec" "real_file.dec"
```
The program is meant for decrypting files, but if you just want to pull a file from in-game and save it as a real file on your actual computer, that is supported with the `--verbatim` option. This might be useful if you want to save a copy of an in-game file for later reference.
Unlike modes where the in-game file must be a valid "dec" file, when `--verbatim` is used, the in-game file can be anything (as long as it exists).


#### Read and decrypt a file named "real_file.dec" that is on your actual computer, printing the decrypted output to the terminal
```
hn-decypher real "real_file.dec"
```
Even when save reading features are enabled when building/installing the program, you can still decrypt actual files that exist on your actual computer using `real` instead of `game`. The example doesn't show this, but you're also able to save decrypted output to an output file by specifying two paths: the first will be the input, and the second will be the output.


### Examples with save reading disabled
If your copy of the program was built/installed without save reading, the command format is simpler: you just feed the program an input file path and optional output file path:

#### Read and decrypt a file named "real_file.dec" that is on your actual computer, printing the decrypted output to the terminal
```
hn-decypher "real_file.dec"
```

#### Read and decrypt a file named "real_file.dec" that is on your actual computer, saving the decrypted output to a file named "decrypted.txt":
```
hn-decypher "real_file.dec" "decrypted.txt"
```

## Notes
- Quotes in the example commands are for clarity and (usually) aren't strictly necessary
- When save reading is enabled, you can explictly specify a non-default profile to read from using the `--account-name` option
- When save reading is enabled, you can explicitly specify a profile-specific XML save file to read from using the `--direct` option
- This program was written on (and tested using) Windows. It *should* also work on Linux and MacOS, but hasn't been tested on either of those platforms. The main difference between the three platforms (that I anticipate affecting this program, at least) is that the default save game location is different for each platform. The program is meant to account for this, but if it ends up pointing to the wrong place, you can explicitly specify what directory your save data is in using `--save-dir`.
  - If you find that the default save game location for a non-Windows platform isn't what this program thinks it is, please open an issue reporting what the actual location was so the program can be fixed and the [PCGamingWiki page](https://www.pcgamingwiki.com/wiki/Hacknet#Save_game_data_location) (from which the current default locations were pulled) can be updated.

