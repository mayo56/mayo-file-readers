use std::f32::consts::PI;

/// Filtre biquad IIR (second ordre)
/// Implémente la forme transposée II (Transposed Direct Form II)
#[derive(Clone, Debug)]
pub struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    // État interne (mémoire du filtre)
    z1: f32,
    z2: f32,
}

impl Biquad {
    /// Crée un filtre peaking EQ (Robert Bristow-Johnson)
    ///
    /// - `sample_rate` : fréquence d'échantillonnage (ex: 44100.0)
    /// - `freq`        : fréquence centrale de la bande (Hz)
    /// - `gain_db`     : gain en décibels (positif = boost, négatif = cut)
    /// - `q`           : facteur de qualité (largeur de bande)
    pub fn peaking_eq(sample_rate: f32, freq: f32, gain_db: f32, q: f32) -> Self {
        let a = 10.0_f32.powf(gain_db / 40.0); // racine carrée du gain linéaire
        let w0 = 2.0 * PI * freq / sample_rate;
        let alpha = w0.sin() / (2.0 * q);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * w0.cos();
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * w0.cos();
        let a2 = 1.0 - alpha / a;

        // Normalisation par a0
        Biquad {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            z1: 0.0,
            z2: 0.0,
        }
    }

    /// Traite un seul échantillon (Transposed Direct Form II)
    #[inline]
    pub fn process_sample(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.z1;
        self.z1 = self.b1 * input - self.a1 * output + self.z2;
        self.z2 = self.b2 * input - self.a2 * output;
        output
    }

    /// Réinitialise l'état interne du filtre
    pub fn reset(&mut self) {
        self.z1 = 0.0;
        self.z2 = 0.0;
    }
}

/// Égaliseur paramétrique multi-bandes
/// Chaîne de filtres biquad en cascade
#[derive(Clone, Debug)]
pub struct Equalizer {
    bands: Vec<Biquad>,
}

/// Paramètres d'une bande de l'égaliseur
pub struct BandParam {
    pub freq: f32,    // Fréquence centrale (Hz)
    pub gain_db: f32, // Gain (dB)
    pub q: f32,       // Facteur de qualité
}

impl Equalizer {
    /// Crée un égaliseur à partir d'une liste de bandes
    pub fn new(sample_rate: f32, bands: &[BandParam]) -> Self {
        Equalizer {
            bands: bands
                .iter()
                .map(|b| Biquad::peaking_eq(sample_rate, b.freq, b.gain_db, b.q))
                .collect(),
        }
    }

    /// Preset "Rock" — courbe en V (bass + treble boost, mid scoop)
    ///
    /// 5 bandes calibrées pour un son rock typique :
    ///   - 80 Hz   → +5.0 dB  (kick, basse)
    ///   - 250 Hz  → +2.0 dB  (corps de la basse/guitare)
    ///   - 1000 Hz → −3.0 dB  (creux médiums, plus d'espace)
    ///   - 3200 Hz → +2.0 dB  (présence guitares, voix)
    ///   - 8000 Hz → +5.0 dB  (brillance cymbales, harmoniques)
    pub fn rock_preset(sample_rate: f32) -> Self {
        // let bands = [
        //     BandParam { freq: 80.0,   gain_db:  5.0, q: 0.8 },
        //     BandParam { freq: 250.0,  gain_db:  2.0, q: 1.0 },
        //     BandParam { freq: 1000.0, gain_db: -3.0, q: 1.2 },
        //     BandParam { freq: 3200.0, gain_db:  2.0, q: 1.0 },
        //     BandParam { freq: 8000.0, gain_db:  5.0, q: 0.8 },
        // ];
        let bands = [
            BandParam { freq: 50.0,    gain_db: 10.0, q: 0.5 },
            BandParam { freq: 200.0,   gain_db:  5.0, q: 0.8 },
            BandParam { freq: 1000.0,  gain_db:  0.0, q: 1.0 },
            BandParam { freq: 3500.0,  gain_db:  4.0, q: 0.9 },
            BandParam { freq: 10000.0, gain_db:  8.0, q: 0.5 },
        ];
        Self::new(sample_rate, &bands)
    }

    /// Traite un buffer d'échantillons en place
    pub fn process(&mut self, samples: &mut [f32]) {
        for sample in samples.iter_mut() {
            for band in self.bands.iter_mut() {
                *sample = band.process_sample(*sample);
            }
        }
    }

    /// Traite un Vec<f32> et retourne un nouveau Vec<f32>
    pub fn process_vec(&mut self, input: &[f32]) -> Vec<f32> {
        let mut output = input.to_vec();
        self.process(&mut output);
        output
    }

    /// Réinitialise tous les filtres
    pub fn reset(&mut self) {
        for band in self.bands.iter_mut() {
            band.reset();
        }
    }
}