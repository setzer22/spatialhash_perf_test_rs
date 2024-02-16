pub mod spatial_hash;
use spatial_hash::*;

use glam::*;

use rand::Rng;

fn main() {
    let mut spatial = SpatialHash::new();

    let mut avg_add = 0.0;
    let mut avg_query = 0.0;
    let mut avg_total = 0.0;
    let mut iterations = 0.0;

    loop {
        let time = std::time::Instant::now();
        let mut rng = rand::thread_rng();

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

        let elapsed_query = time.elapsed();

        iterations += 1.0;
        avg_add += elapsed_add.as_micros() as f64;
        avg_query += elapsed_query.as_micros() as f64;
        avg_total += time.elapsed().as_micros() as f64;

        println!(
            "C: {count} A: {:.2}us ... Q: {:.2}us .. T: {:.2}us",
            avg_add / iterations,
            (avg_query - avg_add) / iterations,
            avg_total / iterations
        );

        spatial.clear();
    }
}
