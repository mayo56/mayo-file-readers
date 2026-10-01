use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

// Types de chunk
#[derive(Clone)]
pub struct RiffHeader {
    pub id: String,
    pub size: u32,
    pub format: String,
}

pub struct FormatHeader {
    pub id: String,
    pub size: u32,
    pub audio_format: u16,
    pub channel_number: u16,
    pub frequency: u32,
    pub byte_per_sec: u32,
    pub byte_per_block: u16,
    pub bits_per_sample: u16,
}

pub struct ListHeader {
    pub id: String,
    pub size: u32,
    pub type_id: String,
    pub data: String,
}

pub struct StartOfChunk {
    pub id: String,
    pub size: u32,
}

pub struct HeaderReader {
    riff_header: Option<RiffHeader>,
    format_header: Option<FormatHeader>,
}

impl HeaderReader {
    pub(crate) fn new() -> Self {
        Self {
            riff_header: None,
            format_header: None,
        }
    }

    /// Lecture de l'en-tête RIFF
    ///
    /// ID      : RIFF
    /// Size    : xxx bytes
    /// Format  : WAVE
    pub fn read_riff_header(&mut self, file: &mut File) -> RiffHeader {
        // Lecture des premiers octets
        (&mut *file).seek(SeekFrom::Start(0)).unwrap();
        let mut buffer: [u8; 12] = [0u8; 12];
        (&mut *file).read_exact(&mut buffer).unwrap();

        RiffHeader {
            id: String::from_utf8_lossy(buffer[0..4].try_into().unwrap())
                .parse()
                .unwrap(),
            size: u32::from_le_bytes(buffer[4..8].try_into().unwrap()),
            format: String::from_utf8_lossy(buffer[8..12].try_into().unwrap())
                .parse()
                .unwrap(),
        }
    }

    /// Lecture de l'en-tête fmt_
    ///
    /// BlockID         : fmt_
    /// Size            : xxx bytes
    /// AudioFormat     : 1, 3, 65534
    /// NbrCanaux       : Nombre de canaux
    /// Fréquence       : 11 025, 22 050, 44 100, 48 000 ou 96 000
    /// BytePerSec      : Fréquence * BytePerBloc
    /// BytePerBloc     : NbrCanaux * BitsPerSample/8
    /// BitsPerSample   : 8, 16, 24, 32
    pub fn read_fmt_header(&mut self, file: &mut File) -> FormatHeader {
        // Buffer sur le header
        let mut buffer: [u8; 24] = [0u8; 24];
        (&mut *file).read_exact(&mut buffer).unwrap();

        FormatHeader {
            id: String::from_utf8_lossy(buffer[0..4].try_into().unwrap())
                .parse()
                .unwrap(),
            size: u32::from_le_bytes(buffer[4..8].try_into().unwrap()),
            audio_format: u16::from_le_bytes(buffer[8..10].try_into().unwrap()),
            channel_number: u16::from_le_bytes(buffer[10..12].try_into().unwrap()),
            frequency: u32::from_le_bytes(buffer[12..16].try_into().unwrap()),
            byte_per_sec: u32::from_le_bytes(buffer[16..20].try_into().unwrap()),
            byte_per_block: u16::from_le_bytes(buffer[20..22].try_into().unwrap()),
            bits_per_sample: u16::from_le_bytes(buffer[22..24].try_into().unwrap()),
        }
    }

    pub fn read_start_chunk(&mut self, file: &mut File) -> StartOfChunk {
        // Buffer du début du chunk
        let mut buffer: [u8; 8] = [0u8; 8];
        (&mut *file).read_exact(&mut buffer).unwrap();
        (&mut *file).seek(SeekFrom::Current(-8)).unwrap();

        StartOfChunk {
            id: String::from_utf8_lossy(&buffer[0..4]).parse().unwrap(),
            size: u32::from_le_bytes(buffer[4..8].try_into().unwrap()),
        }
    }

    pub fn skip_chunk(&mut self, file: &mut File, seek: i64) {
        (&mut *file).seek(SeekFrom::Current(seek)).unwrap();
    }

    pub fn read_list_header(&mut self, file: &mut File) -> ListHeader {
        let header_info: StartOfChunk = (&mut *self).read_start_chunk(file);
        (&mut *file).seek(SeekFrom::Current(8)).unwrap();
        let mut buffer: Vec<u8> = vec![0u8; header_info.size as usize];
        (&mut *file).read_exact(&mut buffer).unwrap();

        ListHeader {
            id: header_info.id,
            size: header_info.size,
            type_id: String::from_utf8_lossy(&buffer[0..4]).parse().unwrap(),
            data: String::from_utf8_lossy(&buffer[4..header_info.size as usize])
                .parse()
                .unwrap(),
        }
    }

    pub fn audio_pattern(&self, code: u16) -> String {
        match code {
            1 => "PCM".to_string(),
            3 => "IEEE".to_string(),
            65534 => "WAVE_FORMAT_EXTENSIBLE".to_string(),
            _ => "Unknow".to_string(),
        }
    }

    pub fn channel_pattern(&self, code: u16) -> String {
        match code {
            1 => "Mono".to_string(),
            2 => "Stéréo".to_string(),
            _ => "Unknow".to_string(),
        }
    }

    // ----------------------------

    pub fn print_header(&mut self, file: &mut File) -> bool {
        println!("═══════════════════════════════════════════════");
        println!("              WAV HEADER ANALYSIS              ");
        println!("═══════════════════════════════════════════════");
        println!();

        let mut header_info: StartOfChunk = (&mut *self).read_start_chunk(file);
        let mut error: bool = false;
        while header_info.id != "data" {
            match header_info.id.as_str() {
                "RIFF" => {
                    (&mut *self).riff_header = Some((&mut *self).read_riff_header(file));
                    (&mut *self).print_riff_header();

                    // Si erreur sur le RIFF
                    if !(&self).validate_riff_header() {
                        println!("[ERROR] RIFF header is not valid");
                        error = true;
                        break;
                    };
                }
                "fmt " => {
                    (&mut *self).format_header = Some((&mut *self).read_fmt_header(file));
                    (&mut *self).print_fmt_header();

                    // Si erreur sur le fmt_
                    if !(&self).validate_fmt_header() {
                        println!("[ERROR] fmt_ header is not valid");
                        error = true;
                        break;
                    };
                }
                "LIST" => {
                    let list_info = (&mut *self).read_list_header(file);
                    (&mut *self).print_list_header(list_info);
                }
                _ => (&mut *self).skip_chunk(file,(header_info.size + 8) as i64),
            };
            header_info = (&mut *self).read_start_chunk(file);
        }
        error
    }

    fn validate_riff_header(&self) -> bool {
        if let Some(data) = &self.riff_header {
            if data.id == "RIFF" && data.format == "WAVE" {
                return true;
            }
        }
        false
    }

    fn validate_fmt_header(&self) -> bool {
        if let Some(data) = &self.format_header {
            let check_byte_per_sec =
                data.frequency * (data.channel_number as u32) * (data.bits_per_sample as u32) / 8;
            if check_byte_per_sec != data.byte_per_sec {
                return false;
            }

            let check_block_align = data.channel_number * data.bits_per_sample / 8;
            if check_block_align != data.byte_per_block {
                return false;
            }
            return true;
        }
        false
    }

    fn print_riff_header(&self) {
        if let Some(data) = &self.riff_header {
            println!("[RIFF Header]");
            println!("  Chunk ID            : {}", data.id);
            println!("  Chunk size          : {} bytes", data.size);
            println!("  Format              : {}", data.format);
            println!();
        }
    }

    fn print_fmt_header(&self) {
        if let Some(data) = &self.format_header {
            println!("[fmt_ Sub-Chunk]");
            println!("  Sub-Chunk ID        : {}", data.id);
            println!("  Sub-Chunk size      : {} bytes", data.size);
            println!(
                "  Audio Format        : {} ({})",
                data.audio_format,
                self.audio_pattern(data.audio_format)
            );
            println!(
                "  Channel             : {} ({})",
                data.channel_number,
                self.channel_pattern(data.channel_number)
            );
            println!("  Sample Rate         : {} Hz", data.frequency);
            println!("  Byte Rate           : {} bytes/sec", data.byte_per_sec);
            println!("  Block Align         : {} bytes", data.byte_per_block);
            println!("  Bits Per Sample     : {} bits", data.bits_per_sample);
            println!();
        }
    }

    fn print_list_header(&self, data: ListHeader) {
        println!("[LIST Header]");
        println!("  Sub-Chunk ID        : {}", data.id);
        println!("  Sub-Chunk Size      : {} bytes", data.size);
        println!("  Sub-Chunk TypeID    : {}", data.type_id);
        println!("  Sub-Chunk Data      : {}", data.data);
        println!();
    }

    // -----
    // Getter/Setter
    // -----
    pub fn get_format_header(self) -> Option<FormatHeader> {
        self.format_header
    }
    pub fn get_riff_header(self) -> Option<RiffHeader> {
        self.riff_header
    }
}
