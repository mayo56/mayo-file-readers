use std::fs::File;
use std::io::Read;
use crate::libs::header_reader::{FormatHeader, HeaderReader, RiffHeader};

pub struct DataReader {
    format_header: FormatHeader
}

impl DataReader {
    pub fn new(header: HeaderReader) -> DataReader {
        DataReader {
            format_header: header.get_format_header().unwrap()
        }
    }

    fn print_data_header(self, id: String, size: u32, byte_per_sec: u32) {
        // Info Chunk
        println!("[DATA Chunk]");
        println!("  ID                  : {}", id);
        println!("  Size                : {} bytes", size);
        println!();

        // Info Music
        let duration = size / byte_per_sec;
        let minutes = duration / 60;
        let seconds = duration % 60 ;
        println!("Duration              : {}s ({}:{})", duration, minutes, seconds);
    }

    pub fn read_sample(self, file: &mut File) -> Vec<f32> {
        let mut buffer_header:[u8;8] = [0u8; 8];
        (&mut *file).read_exact(&mut buffer_header).unwrap();

        let format_header = &self.format_header;

        // Parsing
        let chunk_id: String = String::from_utf8_lossy(&buffer_header[0..4]).to_string();
        let chunk_size: u32 = u32::from_le_bytes(buffer_header[4..8].try_into().unwrap());
        let bps = format_header.byte_per_sec;
        self.print_data_header(chunk_id.clone(), chunk_size, bps);

        let mut result: Vec<f32> = Vec::with_capacity(chunk_size as usize);

        // Verifications
        if chunk_id != "data".to_string() {
            println!("[ERROR] {} is not data chunk", chunk_id);
            return result;
        }

        // ----
        // Lecture PCM
        println!("Lecture des échantillons...");

        let mut count: u32 = 0;
        while count < chunk_size {
            let mut buffer:[u8;4] = [0u8; 4];
            (&mut *file).read_exact(&mut buffer).unwrap();

            let left = i16::from_le_bytes(buffer[0..2].try_into().unwrap());
            let right = i16::from_le_bytes(buffer[2..4].try_into().unwrap());

            (&mut result).push(left as f32 / 32768.0);
            (&mut result).push(right as f32 / 32768.0);
            count += 4;
        }

        println!("Parsing terminé !");

        result
    }
}