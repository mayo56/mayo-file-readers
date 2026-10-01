use std::fs::File;
use std::num::NonZero;
use rodio::{buffer::SamplesBuffer, Player, DeviceSinkBuilder};

mod libs;
use libs::{data_reader::DataReader, header_reader::HeaderReader};
use crate::libs::rock_eq::Equalizer;

fn main() {
    // 1. Ouvrir le fichier
    let mut file: File = File::open("/Users/mayo/Desktop/kiraira.wav")
        .expect("Impossible to open the audio file");

    // 2. Header
    let mut header_reader = HeaderReader::new();

    if !(&mut header_reader).print_header(&mut file) {
        // 3. Data
        let data_reader = DataReader::new(header_reader);
        let sample: Vec<f32> = data_reader.read_sample(&mut file);

        let (left, right): (Vec<f32>, Vec<f32>) = sample
            .chunks(2)
            .map(|c| (c[0], c[1]))
            .unzip();

        // Rock EQ
        let mut eq_l = Equalizer::rock_preset(44100.0);
        let mut eq_r = Equalizer::rock_preset(44100.0);
        let sample_l = eq_l.process_vec(&left);
        let sample_r = eq_r.process_vec(&right);

        let handle = DeviceSinkBuilder::open_default_sink()
            .expect("Impossible d'ouvrir le device audio");
        let player = Player::connect_new(&handle.mixer());

        let output: Vec<f32> = sample_l.iter()
            .zip(sample_r.iter())
            .flat_map(|(&l, &r)| [l, r])
            .collect();

        let source = SamplesBuffer::new(
            NonZero::new(2).unwrap(),
            NonZero::new(44100).unwrap(),
            output
        );

        player.append(source);
        player.sleep_until_end();
    };
}
