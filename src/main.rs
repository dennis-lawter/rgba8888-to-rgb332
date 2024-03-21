mod converter;
mod lookup;

use std::{fs, path::Path};

use clap::{arg, crate_name, crate_version, Command};

fn main() {
    let cmd = Command::new(crate_name!())
        .version(crate_version!())
        .about("A command line tool for converting images from rgba8888 to rgb332")
        .arg(arg!(-i --input <IN_FOLDER> "Folder containing the source rgba8888 sprites as PNG files"))
        .arg(arg!(-o --output <OUT_FOLDER> "Folder to store the output rgb332 files as rust structs"))
        .arg(arg!(-m --mask <MASK_COLOR> "(Optional) Binary rgb332 color representing reserved transparency mask (ex: 0b11100011 or 0b111_000_11)"))
        .arg(arg!(-b --monochrome "(Optional) Output the file as a black & white 1-bit raw file"))
        .get_matches();

    let in_folder = cmd
        .get_one::<String>("input")
        .expect("IN_FOLDER must be provided, please see --help");
    let out_folder = cmd
        .get_one::<String>("output")
        .expect("OUT_FOLDER must be provided, please see --help");
    let mask_opt = cmd.get_one::<String>("mask");
    let black_and_white_flag = cmd.get_flag("monochrome");

    let in_path = Path::new(in_folder);
    assert!(in_path.exists(), "The IN_FOLDER could not be found");
    let out_path = Path::new(out_folder);
    if !out_path.exists() {
        fs::create_dir(out_path)
            .expect("The OUT_FOLDER did not exist, and the directory could not be created");
    }
    let mask_u8_opt = match mask_opt {
        Some(mask_string) => {
            let mask_string_clone = mask_string.clone().replace("_", "");
            assert_eq!(
                mask_string_clone.len(),
                10,
                "Please provide 8 binary digits for the MASK_COLOR, in the format: 0b11100011 or 0b111_000_11"
            );
            let (prefix, mask_string_binary_part) = mask_string_clone.split_at(2);
            assert_eq!(
                prefix, "0b",
                "Please provide 8 binary digits for the MASK_COLOR, in the format 0b11100011"
            );
            Some(
                u8::from_str_radix(&mask_string_binary_part, 2)
                    .expect("Please provide 8 binary digits for the MASK_COLOR"),
            )
        }
        None => None,
    };
    match black_and_white_flag {
        true => converter::convert_png_to_bw(in_path, out_path),
        false => converter::convert_png_to_rgb332(in_path, out_path, mask_u8_opt),
    }
}
