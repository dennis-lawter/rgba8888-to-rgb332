use image::{io::Reader as ImageReader, DynamicImage, GenericImageView};
use std::{
    fs::{self, DirEntry, ReadDir},
    io::{Error, Write},
    path::Path,
};

use crate::lookup::LookupTable;

fn load_dir(in_path: &Path) -> ReadDir {
    fs::read_dir(in_path).expect("An issue occurred enumerating the files in the IN_FOLDER")
}

fn load_img_from_file_result(
    in_file_result: Result<DirEntry, Error>,
) -> Result<(DynamicImage, DirEntry), String> {
    let in_file = match in_file_result {
        Ok(in_file) => in_file,
        Err(err) => {
            return Err(format!("Skipping unreadable file: {}", err));
        }
    };
    match in_file.file_type() {
        Ok(in_file_type) => in_file_type,
        Err(_) => {
            return Err(format!(
                "Skipping file: {}\nError loading metadata",
                in_file.path().display()
            ));
        }
    };

    let image_reader = match ImageReader::open(in_file.path()) {
        Ok(image_reader) => image_reader,
        Err(_) => {
            return Err(format!(
                "Skipping file: {}\nError reading as image",
                in_file.path().display()
            ));
        }
    };

    let image = match image_reader.decode() {
        Ok(image) => image,
        Err(_) => {
            return Err(format!(
                "Skipping file: {}\nError decoding as PNG format",
                in_file.path().display()
            ));
        }
    };

    Ok((image, in_file))
}

pub fn convert_png_to_rgb332(in_path: &Path, out_path: &Path, mask_u8_opt: Option<u8>) -> () {
    let lookup_table = LookupTable::new();
    let in_read_dir = load_dir(in_path);

    for in_file_result in in_read_dir {
        let (image, in_file) = match load_img_from_file_result(in_file_result) {
            Ok(image) => image,
            Err(err_string) => {
                eprintln!("{}", err_string);
                continue;
            }
        };
        let w: usize = image.width() as usize;
        let h: usize = image.height() as usize;
        let x_offset = 0;
        let y_offset = 0;
        let mut output_buffer = vec![0b000_000_00u8; w * h];

        let in_file_name = in_file.file_name();
        let in_file_name_str = in_file_name.to_str().unwrap();

        for y in y_offset..h - y_offset {
            for x in x_offset..w - x_offset {
                let x_u32: u32 = x as u32;
                let y_u32: u32 = y as u32;
                let pixel = image.get_pixel(x_u32, y_u32);
                let pixel_bytes = pixel.0;
                match lookup_table.get(&pixel_bytes) {
                    Ok(Some(byte)) => output_buffer[y * w + x] = byte,
                    Ok(None) => match mask_u8_opt {
                        Some(mask) => {
                            output_buffer[y * w + x] = mask;
                        }
                        None => {
                            let r_ch: u32 = pixel_bytes[0].into();
                            let g_ch: u32 = pixel_bytes[1].into();
                            let b_ch: u32 = pixel_bytes[2].into();
                            let a_ch: u32 = pixel_bytes[3].into();
                            let pixel_full_uint: u32 = r_ch << 24 ^ g_ch << 16 ^ b_ch << 8 ^ a_ch;
                            panic!(
                                "Transparency is not supported without the --mask flag!\nA transparent pixel ({:#010x}) was found in file {}",
                                pixel_full_uint, in_file_name_str
                            );
                        }
                    },
                    Err(_) => {
                        let r_ch: u32 = pixel_bytes[0].into();
                        let g_ch: u32 = pixel_bytes[1].into();
                        let b_ch: u32 = pixel_bytes[2].into();
                        let a_ch: u32 = pixel_bytes[3].into();
                        let pixel_full_uint: u32 = r_ch << 24 ^ g_ch << 16 ^ b_ch << 8 ^ a_ch;
                        panic!(
                            "An unsupported pixel ({:#010x}) was found in file {}",
                            pixel_full_uint, in_file_name_str
                        );
                    }
                }
            }
        }

        let out_file_name = format!("{}.data", in_file_name_str.replace(".png", ""));
        let out_file_name_str = out_file_name.as_str();

        let full_out_file_name = out_path.join(out_file_name_str);

        let mut out_file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&full_out_file_name)
            .expect(
                format!(
                    "Fatal error, cannot open file for writing: {}",
                    &out_file_name_str
                )
                .as_str(),
            );
        out_file
            .write_all(&output_buffer)
            .expect(format!("Fatal error, cannot write to file: {}", &out_file_name_str).as_str());

        println!("Converted {} to {}", in_file_name_str, out_file_name_str);
    }
}

pub fn convert_png_to_bw(in_path: &Path, out_path: &Path) -> () {
    let in_read_dir = load_dir(in_path);

    for in_file_result in in_read_dir {
        let (image, in_file) = match load_img_from_file_result(in_file_result) {
            Ok(image) => image,
            Err(err_string) => {
                eprintln!("{}", err_string);
                continue;
            }
        };
        let w: usize = image.width() as usize;
        let h: usize = image.height() as usize;
        if w % 8 != 0 {
            eprintln!(
                "Skipping file: {}\nBlack & White output files must have a width divisible by 8",
                in_file.path().display()
            );
            continue;
        }
        let x_offset = 0;
        let y_offset = 0;
        let mut output_buffer = vec![0b000_000_00u8; (w / 8) * h];

        println!("SIZE: {} x {} = {} bytes", w, h, output_buffer.len());

        let in_file_name = in_file.file_name();
        let in_file_name_str = in_file_name.to_str().unwrap();

        for y in y_offset..h - y_offset {
            for x in x_offset..w - x_offset {
                let x_u32: u32 = x as u32;
                let y_u32: u32 = y as u32;
                let pixel = image.get_pixel(x_u32, y_u32);
                let pixel_bytes = pixel.0;
                match pixel_bytes {
                    [0x00, 0x00, 0x00, 0xff] => {
                        continue;
                    }
                    [0xff, 0xff, 0xff, 0xff] => {
                        let offset_in_byte = x % 8;
                        let byte_index = (y * (w / 8)) + (x / 8);
                        let bit_set = 0b1000_0000 >> offset_in_byte;
                        output_buffer[byte_index] ^= bit_set;
                    }
                    invalid_pixel => {
                        let r_ch: u32 = invalid_pixel[0].into();
                        let g_ch: u32 = invalid_pixel[1].into();
                        let b_ch: u32 = invalid_pixel[2].into();
                        let a_ch: u32 = invalid_pixel[3].into();
                        let pixel_full_uint: u32 = r_ch << 24 ^ g_ch << 16 ^ b_ch << 8 ^ a_ch;
                        panic!(
                            "An unsupported pixel ({:#010x}) was found in file {}",
                            pixel_full_uint, in_file_name_str
                        );
                    }
                }
            }
        }

        let out_file_name = format!("{}.data", in_file_name_str.replace(".png", ""));
        let out_file_name_str = out_file_name.as_str();

        let full_out_file_name = out_path.join(out_file_name_str);

        let mut out_file = fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&full_out_file_name)
            .expect(
                format!(
                    "Fatal error, cannot open file for writing: {}",
                    &out_file_name_str
                )
                .as_str(),
            );

        println!("SIZE: {} x {} = {} bytes", w, h, output_buffer.len());
        out_file
            .write_all(&output_buffer)
            .expect(format!("Fatal error, cannot write to file: {}", &out_file_name_str).as_str());
        println!("FINAL FILESIZE: {}", out_file.metadata().unwrap().len());

        println!("Converted {} to {}", in_file_name_str, out_file_name_str);
    }
}
