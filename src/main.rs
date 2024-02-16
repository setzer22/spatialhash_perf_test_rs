pub mod spatial_hash;

use spatial_hash::*;
use std::time::Duration;

use glam::*;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

const WARMUP_ITERS: usize = 0;
const MEASURE_ITERS: usize = 700;

fn main() {
    let mut spatial = SpatialHash::new();

    let mut min_add = None;
    let mut min_query = None;

    let mut median_add = Vec::new();
    let mut median_query = Vec::new();

    let mut iterations = 0;
    let seed = 12345;
    let mut rng = StdRng::seed_from_u64(seed);

    // Note: since rng calls C, might be non-portable, and you might want to adjust results on your machine.
    let expected_results = [52696, 55359, 53132, 52800, 54877];

    loop {
        let time = std::time::Instant::now();

        let mk_circle = |pos: Vec2| {
            Shape::Circle(CircleShape {
                center: pos.into(),
                radius: 2.0,
            })
        };

        let mk_aabb = |pos: Vec2| {
            Shape::Aabb(AabbShape {
                min: (Vec2 {
                    x: pos.x - 2.0,
                    y: pos.y - 2.0,
                }),
                max: (Vec2 {
                    x: pos.x + 2.0,
                    y: pos.x + 2.0,
                }),
            })
        };

        for i in 0..2000 {
            let pos = Vec2 {
                x: rng.gen_range::<f32, _>(-60.0..60.0),
                y: rng.gen_range::<f32, _>(-60.0..60.0),
            };

            if rng.gen::<f32>() < 0.5 {
                spatial.add_shape(
                    mk_circle(pos),
                    SpatialUserData {
                        entity_type: 1,
                        entity_id: i,
                    },
                )
            } else {
                spatial.add_shape(
                    mk_aabb(pos),
                    SpatialUserData {
                        entity_type: 1,
                        entity_id: i,
                    },
                )
            }
        }

        let elapsed_add = time.elapsed();

        let mut count = 0;
        for _ in 0..2000 {
            let pos = Vec2 {
                x: rng.gen_range::<f32, _>(-60.0..60.0),
                y: rng.gen_range::<f32, _>(-60.0..60.0),
            };

            let shape = if rng.gen::<f32>() < 0.5 {
                mk_circle(pos)
            } else {
                mk_aabb(pos)
            };

            let result = spatial.query(SpatialQuery::ShapeQuery(shape));
            count += result.count();
        }

        iterations += 1usize;
        if iterations >= WARMUP_ITERS {
            let elapsed_query = time.elapsed();
            //
            // avg_add += elapsed_add.as_micros() as f64;
            // avg_query += elapsed_query.as_micros() as f64;
            // avg_total += time.elapsed().as_micros() as f64;

            min_add = min_add
                .map(|x| Duration::min(x, elapsed_add))
                .or(Some(elapsed_add));
            min_query = min_query
                .map(|x| Duration::min(x, elapsed_query))
                .or(Some(elapsed_query));

            // Compute median
            median_add.push(elapsed_add);
            median_query.push(elapsed_query);

            // End
            if iterations >= MEASURE_ITERS + 5 {
                break;
            }

            // Last iteration
            if iterations == MEASURE_ITERS + 4 {
                median_add.sort();
                median_query.sort();

                println!(
                    "Min:      A: {:.2}us    Q: {:.2}us",
                    min_add.unwrap().as_micros() as f64,
                    min_query.unwrap().as_micros() as f64,
                );

                println!(
                    "Median:   A: {:.2}us    Q: {:.2}us",
                    median_add[MEASURE_ITERS / 2].as_micros() as f64,
                    median_query[MEASURE_ITERS / 2].as_micros() as f64,
                );
            }

            if iterations >= MEASURE_ITERS {
                // Check deterministic results, to avoid buge
                assert_eq!(
                    count,
                    expected_results[iterations - MEASURE_ITERS],
                    "BLIN! iz buge (or other RNG)"
                );

                // println!("Results:   C: {count}");
            }
        }

        spatial.clear();
    }
}
