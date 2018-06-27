use std::marker::PhantomData;

use images::{MultipleOf, Size, SizedImage, SmallerThan};
use post::{GenericPost, Post};
use super::MosaicPiece;

pub type Distance = u64;

pub struct DistanceCalcAlgo<S, SS> {
    algo: MeanGrayscaleAlgo,
    _piece_size: PhantomData<(S, SS)>,
}

impl<S, SS> DistanceCalcAlgo<S, SS>
where
    S: Size + MultipleOf<SS>,
    SS: Size + SmallerThan<S>,
{
    pub fn new(origin: &SizedImage<S>) -> DistanceCalcAlgo<S, SS> {
        let algo = MeanGrayscaleAlgo::new(&origin);
        DistanceCalcAlgo {
            algo: algo,
            _piece_size: PhantomData,
        }
    }

    pub fn calc_post(&self, post: GenericPost<SS>) -> MosaicPiece<SS> {
        let vec = self.algo.calc(&post.image());
        MosaicPiece {
            post: post,
            distance_vec: vec,
        }
    }
}

struct MeanGrayscaleAlgo {
    // Cache of origin piece's mean grayscale
    cache: Vec<f64>,
}

impl MeanGrayscaleAlgo {
    fn new<S, SS>(origin: &SizedImage<S>) -> MeanGrayscaleAlgo
    where
        S: Size + MultipleOf<SS>,
        SS: Size + SmallerThan<S>,
    {
        let cache = origin
            .split_into_pieces()
            .map(|p| p.image.mean_grayscale())
            .collect();
        MeanGrayscaleAlgo { cache: cache }
    }

    fn calc<SS: Size>(&self, piece: &SizedImage<SS>) -> Vec<Distance> {
        let mean = piece.mean_grayscale();
        self.cache
            .iter()
            .map(move |f| (f64::abs(f - mean) * 10000f64) as u64)
            .collect()
    }
}
