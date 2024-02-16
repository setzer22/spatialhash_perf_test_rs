use fxhash::{FxHashMap, FxHashSet};
use glam::*;

#[derive(Clone, Copy, Debug)]
pub struct AabbShape {
    pub min: Vec2,
    pub max: Vec2,
}

impl AabbShape {
    pub fn intersects_circle(&self, circle: CircleShape) -> bool {
        let closest = self.min.max(self.max.min(circle.center));
        let distance = circle.center.distance(closest);
        distance <= circle.radius
    }

    pub fn intersects_aabb(&self, aabb: AabbShape) -> bool {
        self.min.x <= aabb.max.x
            && self.max.x >= aabb.min.x
            && self.min.y <= aabb.max.y
            && self.max.y >= aabb.min.y
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CircleShape {
    pub center: Vec2,
    pub radius: f32,
}

impl CircleShape {
    pub fn bounding_rect(&self) -> AabbShape {
        let min = self.center - Vec2::splat(self.radius);
        let max = self.center + Vec2::splat(self.radius);
        AabbShape { min, max }
    }

    pub fn intersects_circle(&self, circle: CircleShape) -> bool {
        let distance = self.center.distance(circle.center);
        distance <= self.radius + circle.radius
    }

    pub fn intersects_aabb(&self, aabb: AabbShape) -> bool {
        aabb.intersects_circle(*self)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Circle(CircleShape),
    Aabb(AabbShape),
}

impl Shape {
    pub fn bounding_rect(&self) -> AabbShape {
        match self {
            Shape::Circle(circle) => circle.bounding_rect(),
            Shape::Aabb(aabb) => *aabb,
        }
    }

    pub fn intersects_shape(&self, shape: Shape) -> bool {
        match (*self, shape) {
            (Shape::Circle(circle1), Shape::Circle(circle2)) => circle1.intersects_circle(circle2),
            (Shape::Circle(circle), Shape::Aabb(aabb))
            | (Shape::Aabb(aabb), Shape::Circle(circle)) => circle.intersects_aabb(aabb),
            (Shape::Aabb(aabb1), Shape::Aabb(aabb2)) => aabb1.intersects_aabb(aabb2),
        }
    }
}

#[derive(Clone, Copy)]
pub enum SpatialQuery {
    ShapeQuery(Shape),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SpatialUserData {
    pub entity_type: u32,
    pub entity_id: u32,
}

#[derive(Clone, Copy)]
pub struct SpatialHashData {
    pub shape: Shape,
    pub userdata: SpatialUserData,
}

pub struct SpatialHash {
    grid_size: f32,
    inner: FxHashMap<(i32, i32), Vec<SpatialHashData>>,
}

impl SpatialHash {
    pub fn new() -> Self {
        const DEFAULT_GRID_SIZE: f32 = 15.0;
        Self {
            grid_size: DEFAULT_GRID_SIZE,
            inner: FxHashMap::default(),
        }
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn add_shape(&mut self, shape: Shape, data: SpatialUserData) {
        let bounding_rect = match shape {
            Shape::Circle(circle) => circle.bounding_rect(),
            Shape::Aabb(aabb) => aabb,
        };

        let min = bounding_rect.min / self.grid_size;
        let max = bounding_rect.max / self.grid_size;
        let min = min.floor();
        let max = max.ceil();
        for x in min.x as i32..max.x as i32 {
            for y in min.y as i32..max.y as i32 {
                let key = (x, y);
                let entry = self.inner.entry(key).or_insert_with(Vec::new);
                entry.push(SpatialHashData {
                    shape,
                    userdata: data,
                });
            }
        }
    }

    pub fn query(&self, query: SpatialQuery) -> impl Iterator<Item = &SpatialUserData> {
        match query {
            SpatialQuery::ShapeQuery(shape) => {
                let bounding_rect = shape.bounding_rect();
                let min = bounding_rect.min / self.grid_size;
                let max = bounding_rect.max / self.grid_size;
                let min = min.floor();
                let max = max.ceil();
                (min.x as i32..max.x as i32)
                    .flat_map(move |x| (min.y as i32..max.y as i32).map(move |y| (x, y)))
                    .flat_map(move |key| self.inner.get(&key).into_iter().flatten())
                    .filter(move |data| data.shape.intersects_shape(shape))
                    .map(|data| &data.userdata)
                    .collect::<FxHashSet<_>>()
                    .into_iter()
            }
        }
    }
}
