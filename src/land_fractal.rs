extern crate noise;

use noise::{NoiseFn, Perlin, Point2, Seedable};


#[derive(Clone, Debug, Default)]
pub struct LandFractal {
    seed: u32,
    sources: Vec<Perlin>,
}


impl LandFractal {
    pub const DEFAULT_SEED: u32 = 0;
    pub const OCTAVES: usize = 8;

    fn build_sources(seed: u32) -> Vec<Perlin> {
        let mut sources = Vec::with_capacity(Self::OCTAVES);

        for x in 0..Self::OCTAVES {
            sources.push(Perlin::new().set_seed(seed + x as u32));
        }

        sources
    }


    pub fn new() -> Self {
        LandFractal { seed: Self::DEFAULT_SEED,
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
        let octaves: i32 = 8;
        let persistence: f64 = 0.5;
        let lacunarity = std::f64::consts::PI * 2.0 / 3.0;
        let mut result;

        // Octave 0 - The basic shape of the terrain
        let base = self.sources[0].get(point);

        result = base;

        // Octave 1
        let point2 =  [point[0] * lacunarity, point[1] * lacunarity];
        let octave1 = self.sources[1].get(point2) * persistence;

        result += octave1;

        // Octave 2
        let point3 =  [point2[0] * lacunarity, point2[1] * lacunarity];
        let mut octave2 = self.sources[2].get(point3);
        octave2 *= persistence.powi(2);

        result += octave2;

        // Octave 3
        let point4 =  [point3[0] * lacunarity, point3[1] * lacunarity];
        let mut octave3 = self.sources[3].get(point4);
        octave3 *= persistence.powi(3);

        result += octave3;

        // Octave 4
        let point5 =  [point4[0] * lacunarity, point4[1] * lacunarity];
        let mut octave4 = self.sources[4].get(point5);
        octave4 *= persistence.powi(4);

        result += octave4;

        // Octave 5
        let point6 =  [point5[0] * lacunarity, point5[1] * lacunarity];
        let mut octave5 = self.sources[5].get(point6);
        octave5 *= persistence.powi(5);

        result += octave5;

        // Octave 6
        let point7 =  [point6[0] * lacunarity, point6[1] * lacunarity];
        let mut octave6 = self.sources[6].get(point7);
        octave6 *= persistence.powi(6);

        result += octave6;

        // Octave 7
        let point8 =  [point7[0] * lacunarity, point7[1] * lacunarity];
        let mut octave7 = self.sources[7].get(point8);
        octave7 *= persistence.powi(7);

        result += octave7;

        let scale = 2.0 - persistence.powi(octaves - 1);
        result / scale
    }
}
