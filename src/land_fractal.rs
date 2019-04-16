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

    fn scale_point(&self, point: Point2<f64>) -> Point2<f64> {
        [point[0] * Self::LACUNARITY, point[1] * Self::LACUNARITY]
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
    fn get(&self, mut point: Point2<f64>) -> f64 {
        let mut result;

        // Octave 0 - The basic shape of the terrain
        let base = self.sources[0].get(point);

        result = base * 1.5;

        // Octave 1 - Large details
        point = self.scale_point(point);
        let octave1 = self.sources[1].get(point) * Self::PERSISTENCE;

        result += octave1;

        // Octave 2 - Large details breakup
        point = self.scale_point(point);
        let mut octave2 = self.sources[2].get(point);
        octave2 *= Self::PERSISTENCE.powi(2);

        result += octave2;

        // Octave 3 - Medium details
        point = self.scale_point(point);
        let mut octave3 = self.sources[3].get(point);
        octave3 *= Self::PERSISTENCE.powi(4);

        result += octave3;

        // Octave 4 - Smaller details
        point = self.scale_point(point);
        let mut octave4 = self.sources[4].get(point);
        octave4 *= Self::PERSISTENCE.powi(6);

        result += octave4;

        // Octave 5 - Fine details
        point = self.scale_point(point);
        let mut octave5 = self.sources[5].get(point);
        octave5 *= Self::PERSISTENCE.powi(7);

        result += octave5;

        (result / self.scale) + 0.5
    }
}
