use std::{
    collections::hash_map::{Entry, HashMap},
    num::NonZeroUsize,
    sync::Arc,
    thread,
};

use once_cell::sync::OnceCell;
use optimal::{optimizer::derivative_free::pbil::*, prelude::*};
use owm_problem::{
    encoding::Decoder, objective::Problem, post_processing::overlap_borders, AreaRatio,
    AspectRatio, Rect, Size, Weights,
};
use rand::prelude::*;
use rand_xoshiro::SplitMix64;
use rayon::prelude::*;

#[derive(Debug)]
pub struct LayoutGen {
    inner: Arc<RawLayoutGen>,
    cache: HashMap<Key, Arc<OnceCell<Vec<Rect>>>>,
}

#[derive(Clone, Debug)]
struct RawLayoutGen {
    min_width: NonZeroUsize,
    min_height: NonZeroUsize,
    max_width: Option<NonZeroUsize>,
    max_height: Option<NonZeroUsize>,
    overlap_borders_by: usize,
    weights: Weights,
    area_ratios: Vec<AreaRatio>,
    aspect_ratios: Vec<AspectRatio>,
}

type Key = (Size, usize);

pub enum Status<'a> {
    NotStarted,
    Started,
    Finished(&'a [Rect]),
}

impl LayoutGen {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        min_width: NonZeroUsize,
        min_height: NonZeroUsize,
        max_width: Option<NonZeroUsize>,
        max_height: Option<NonZeroUsize>,
        overlap_borders_by: usize,
        weights: Weights,
        area_ratios: Vec<AreaRatio>,
        aspect_ratios: Vec<AspectRatio>,
    ) -> Self {
        Self {
            inner: Arc::new(RawLayoutGen {
                min_width,
                min_height,
                max_width,
                max_height,
                overlap_borders_by,
                weights,
                area_ratios,
                aspect_ratios,
            }),
            cache: HashMap::new(),
        }
    }

    pub fn try_layout(&self, container: Size, count: usize) -> Status {
        match self.cache.get(&(container, count)) {
            Some(cache_cell) => match cache_cell.get() {
                Some(layout) => Status::Finished(layout),
                None => Status::Started,
            },
            None => Status::NotStarted,
        }
    }

    pub fn layout<F>(&mut self, container: Size, count: usize, callback: F)
    where
        F: FnOnce(&[Rect]) + Send + 'static,
    {
        self._layout(container, count, Box::new(callback))
    }

    // `Box` avoids infinite recusion during compilation.
    #[allow(clippy::type_complexity)]
    fn _layout(
        &mut self,
        container: Size,
        count: usize,
        callback: Box<dyn FnOnce(&[Rect]) + Send + 'static>,
    ) {
        let key = (container, count);
        if count == 0 {
            return (callback)(
                self.cache
                    .entry(key)
                    .or_insert(Arc::new(OnceCell::new()))
                    .get_or_init(Vec::new),
            );
        }
        match self.cache.entry(key) {
            Entry::Vacant(entry) => {
                let cache_cell = Arc::clone(entry.insert(Arc::new(OnceCell::new())));
                let gen = Arc::clone(&self.inner);
                self.layout(
                    container,
                    count - 1,
                    Box::new(move |prev_layout: &[Rect]| {
                        let prev_layout = prev_layout.to_vec();
                        thread::spawn(move || {
                            let layout = gen.layout(container, prev_layout);
                            let layout = cache_cell
                                .try_insert(layout)
                                .expect("cell should be unset for {key:?}");
                            (callback)(layout)
                        });
                    }),
                );
            }
            Entry::Occupied(entry) => {
                let cache_cell = entry.get();
                if let Some(layout) = cache_cell.get() {
                    (callback)(layout)
                } else {
                    let cache_cell = Arc::clone(cache_cell);
                    thread::spawn(move || (callback)(cache_cell.wait()));
                }
            }
        }
    }
}

impl RawLayoutGen {
    fn layout(&self, container: Size, prev_layout: Vec<Rect>) -> Vec<Rect> {
        let count = prev_layout.len() + 1;
        let max_size = Size::new(
            self.max_width
                .map_or(container.width, |x| x.min(container.width)),
            self.max_height
                .map_or(container.height, |x| x.min(container.height)),
        );
        let decoder = Decoder::new(
            Size::new(
                self.min_width.min(container.width),
                self.min_height.min(container.height),
            ),
            max_size,
            container,
            count,
        );
        let problem = Problem::new(
            self.weights,
            self.area_ratios.clone(),
            self.aspect_ratios.clone(),
            max_size,
            container,
            prev_layout,
        );
        let mut rects = decoder
            .decode1(
                UntilConvergedConfig {
                    threshold: ProbabilityThreshold::new(Probability::new(0.9).unwrap()).unwrap(),
                }
                .argmin(
                    &mut Config {
                        num_samples: NumSamples::new(
                            500 * std::thread::available_parallelism().map_or(1, |x| x.into()),
                        )
                        .unwrap(),
                        adjust_rate: AdjustRate::new(0.1).unwrap(),
                        mutation_chance: MutationChance::new(0.0).unwrap(),
                        mutation_adjust_rate: MutationAdjustRate::new(0.05).unwrap(),
                    }
                    .start_using(
                        decoder.bits(),
                        |points| {
                            (0..points.nrows())
                                .into_par_iter()
                                .map(|i| {
                                    problem.evaluate(
                                        decoder.decode1(points.row(i)).as_slice().unwrap(),
                                    )
                                })
                                .collect::<Vec<_>>()
                                .into()
                        },
                        &mut SplitMix64::seed_from_u64(0),
                    ),
                )
                .view(),
            )
            .into_raw_vec();
        if self.overlap_borders_by > 0 {
            overlap_borders(self.overlap_borders_by, container, &mut rects);
        }
        rects
    }
}
