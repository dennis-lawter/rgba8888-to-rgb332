use image::{io::Reader as ImageReader, GenericImageView};
use std::{fs, io::Write, path::Path};

use crate::lookup::LookupTable;

pub fn convert(in_path: &Path, out_path: &Path, mask_u8_opt: Option<u8>) -> () {
    let lookup_table = LookupTable::new();
    let in_read_dir =
        fs::read_dir(in_path).expect("An issue occurred enumerating the files in the IN_FOLDER");

    for in_file_result in in_read_dir {
        let in_file = match in_file_result {
            Ok(in_file) => in_file,
            Err(err) => {
                println!("Skipping unreadable file: {}", err);
                continue;
            }
        };
        match in_file.file_type() {
            Ok(in_file_type) => in_file_type,
            Err(_) => {
                println!(
                    "Skipping file: {}\nError loading metadata",
                    in_file.path().display()
                );
                continue;
            }
        };

        let image_reader = match ImageReader::open(in_file.path()) {
            Ok(image_reader) => image_reader,
            Err(_) => {
                println!(
                    "Skipping file: {}\nError reading as image",
                    in_file.path().display()
                );
                continue;
            }
        };

        let image = match image_reader.decode() {
            Ok(image) => image,
            Err(_) => {
                println!(
                    "Skipping file: {}\nError decoding as PNG format",
                    in_file.path().display()
                );
                continue;
            }
        };

        let w: usize = image
            .width()
            .try_into()
            .expect("Fatal error parsing image width");
        let h: usize = image
            .height()
            .try_into()
            .expect("Fatal error parsing image height");
        // let x_offset = if w > h { (w - h) / 2 } else { 0 };
        // let y_offset = if h > w { (h - w) / 2 } else { 0 };
        let x_offset = 0;
        let y_offset = 0;;;;;;;/mnnnnn ,l.
        let mut output_buffer = vec![0b000_000_00u8; w * h];

        let in_file_name = in_file.file_name();
        let in_file_name_str = in_file_name.to_str().unwrap();

        // println!(
        //     "Image is {} x {}, offsets are {} x {}",
        //     w, h, x_offset, y_offset
        // );

        for y in y_offset..h - y_offset {
            for x in x_offset..w - x_offset {
                let x_u32: u32 = x.try_into().expect("Fatal error parsing cursor X");
                let y_u32: u32 = y.try_into().expect("Fatal error parsing cursor Y");
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
                                "Transparenty is not supported without the --mask flag!\nA transparent pixel ({:#010x}) was found in file {}",
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

        let out_file_name = format!("{}.data", in_file_name_str);
        let out_file_name_str = out_file_name.as_str();

        let full_out_file_name = out_path.join(out_file_name_str);

        let mut out_file = fs::OpenOptions::new()
            .create(true)
            .write(true)
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
