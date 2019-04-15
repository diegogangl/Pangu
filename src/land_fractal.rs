extern crate noise;

use noise::{NoiseFn, Perlin, Point2, Seedable};


#[derive(Clone, Debug, Default)]
pub struct LandFractal {
    seed: u32,
    scale: f64,
    sources: Vec<Perlin>,
}


impl LandFractal {
    pub const DEFAULT_SEED: u32 = 0;

    const OCTAVES: usize = 6;
    const LACUNARITY: f64 = std::f64::consts::PI * 2.0 / 3.0;
    const PERSISTENCE: f64 = 0.5;


    fn build_sources(seed: u32) -> Vec<Perlin> {
        let mut sources = Vec::with_capacity(Self::OCTAVES);

        for x in 0..Self::OCTAVES {
            sources.push(Perlin::new().set_seed(seed + x as u32));
        }

        sources
    }


    pub fn new() -> Self {
        LandFractal { seed: Self::DEFAULT_SEED,
                      scale: 2.0 - Self::PERSISTENCE.powi(Self::OCTAVES as i32 - 1),
                      sources: Self::build_sources(Self::DEFAULT_SEED) }
    }
}


impl Seedable for LandFractal {
    fn set_seed(self, seed: u32) -> Self {
        if self.seed == seed {
            return self;
        }

        LandFractal { seed: seed,
                      sources: Self::build_sources(Self::DEFAULT_SEED),
                      ..self }
    }

    fn seed(&self) -> u32 {
        self.seed
    }
}



impl NoiseFn<Point2<f64>> for LandFractal {
    fn get(&self, point: Point2<f64>) -> f64 {
        let mut result;

        // Octave 0 - The basic shape of the terrain
        let base = self.sources[0].get(point);

        result = base;

        // Octave 1 - Large details
        let point2 =  [point[0] * Self::LACUNARITY, point[1] * Self::LACUNARITY];
        let octave1 = self.sources[1].get(point2) * Self::PERSISTENCE;

        result += octave1;

        // Octave 2 - Large details breakup
        let point3 =  [point2[0] * Self::LACUNARITY, point2[1] * Self::LACUNARITY];
        let mut octave2 = self.sources[2].get(point3);
        octave2 *= Self::PERSISTENCE.powi(2);

        result += octave2;

        // Octave 3 - Medium details
        let point4 =  [point3[0] * Self::LACUNARITY, point3[1] * Self::LACUNARITY];
        let mut octave3 = self.sources[3].get(point4);
        octave3 *= Self::PERSISTENCE.powi(3);

        result += octave3;

        // Octave 4 - Smaller details
        let point5 =  [point4[0] * Self::LACUNARITY, point4[1] * Self::LACUNARITY];
        let mut octave4 = self.sources[4].get(point5);
        octave4 *= Self::PERSISTENCE.powi(4);

        result += octave4;

        // Octave 5 - Fine details
        let point6 =  [point5[0] * Self::LACUNARITY, point5[1] * Self::LACUNARITY];
        let mut octave5 = self.sources[5].get(point6);
        octave5 *= Self::PERSISTENCE.powi(5);

        result += octave5;

        (result / self.scale) + 0.5
    }
}
