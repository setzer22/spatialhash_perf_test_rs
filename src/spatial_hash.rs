use fxhash::{FxHashMap, FxHashSet};
use glam::*;

#[derive(Clone, Copy, Debug)]
pub struct AabbShape {
    pub min: Vec2,
    pub max: Vec2,
}

#[derive(Clone, Copy, Debug)]
pub struct CircleShape {
    pub center: Vec2,
    pub radius: f32,
}

impl CircleShape {
    pub fn bounding_rect(&self) -> AabbShape {
        let rr = Vec2::splat(self.radius);
        let min = self.center - rr;
        let max = self.center + rr;
        AabbShape { min, max }
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

    pub fn as_circle(&self) -> &CircleShape {
        match self {
            Shape::Circle(circle) => circle,
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }

    pub fn as_aabb(&self) -> &AabbShape {
        match self {
            Shape::Aabb(aabb) => aabb,
            _ => unsafe { std::hint::unreachable_unchecked() },
        }
    }
}

pub fn intersect_aabb_circle(aabb: &AabbShape, circle: &CircleShape) -> bool {
    let closest = Vec2::max(aabb.min, Vec2::min(aabb.max, circle.center));

    let dist_sq = circle.center.distance_squared(closest);
    dist_sq <= circle.radius * circle.radius
}

pub fn intersect_aabb_aabb(a_left: &AabbShape, a_right: &AabbShape) -> bool {
    // a_left.min.x <= a_right.max.x
    //     && a_left.min.y <= a_right.max.y
    //     && a_left.max.x >= a_right.min.x
    //     && a_left.max.y >= a_right.min.y

    a_left.min.cmple(a_right.max).all() && a_left.max.cmpge(a_right.min).all()
}
pub fn intersect_circle_circle(c_left: &CircleShape, c_right: &CircleShape) -> bool {
    let distance = c_left.center.distance_squared(c_right.center);
    let both = c_left.radius + c_right.radius;
    distance <= both * both
}

#[derive(Clone, Copy)]
pub enum SpatialQuery {
    ShapeQuery(Shape),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct SpatialUserData {
    pub entity_type: u32,
    pub entity_id: u32,
}

impl SpatialUserData {
    pub fn linearize(&self) -> u32 {
        // Use entity-type as top-most bit
        (self.entity_type << 16) | self.entity_id
    }

    pub fn from_linearized(linearized: u32) -> Self {
        let entity_type = linearized >> 16;
        let entity_id = linearized & 0xFFFF;
        Self {
            entity_type,
            entity_id,
        }
    }
}

#[derive(Clone, Copy)]
pub struct SpatialHashData {
    pub shape: Shape,
    pub userdata: SpatialUserData,
}

#[derive(Default)]
pub struct Cell {
    pub circles: Vec<SpatialHashData>,
    pub aabbs: Vec<SpatialHashData>,
}

pub struct SpatialHash {
    grid_size: f32,
    inner: FxHashMap<(i32, i32), Cell>,
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
                let cell = self.inner.entry(key).or_insert_with(Cell::default);

                let vec = match shape {
                    Shape::Circle(_) => &mut cell.circles,
                    Shape::Aabb(_) => &mut cell.aabbs,
                };

                vec.push(SpatialHashData {
                    shape,
                    userdata: data,
                });
            }
        }
    }

    pub fn query<'a, 'b: 'a>(
        &'b self,
        query: SpatialQuery,
        // out_vec: &mut FxHashSet<&'a SpatialUserData>,
        out_vec: &mut FxHashSet<u32>,
        // out_vec: &mut BitSet
    )
    /*-> impl Iterator<Item = &SpatialUserData>*/
    {
        match query {
            SpatialQuery::ShapeQuery(shape) => {
                let bounding_rect = shape.bounding_rect();
                let min = bounding_rect.min / self.grid_size;
                let max = bounding_rect.max / self.grid_size;
                let min = min.floor();
                let max = max.ceil();

                out_vec.clear();
                // Loop over the range of x values.
                for x in min.x as i32..max.x as i32 {
                    // Nested loop over the range of y values.
                    for y in min.y as i32..max.y as i32 {
                        let key = (x, y); // Create a key from the current x and y values.

                        // Attempt to retrieve the value associated with the current key from `self.inner`.
                        if let Some(cell) = self.inner.get(&key) {
                            // If data exists for the key, iterate over the elements.
                            for data in cell.aabbs.iter() {
                                match shape {
                                    Shape::Circle(circle) => {
                                        if intersect_aabb_circle(data.shape.as_aabb(), &circle) {
                                            out_vec.insert(data.userdata.linearize());
                                        }
                                    }
                                    Shape::Aabb(aabb) => {
                                        if intersect_aabb_aabb(data.shape.as_aabb(), &aabb) {
                                            out_vec.insert(data.userdata.linearize());
                                        }
                                    }
                                }
                            }

                            for data in cell.circles.iter() {
                                match shape {
                                    Shape::Circle(circle) => {
                                        if intersect_circle_circle(data.shape.as_circle(), &circle)
                                        {
                                            out_vec.insert(data.userdata.linearize());
                                        }
                                    }
                                    Shape::Aabb(aabb) => {
                                        if intersect_aabb_circle(&aabb, data.shape.as_circle()) {
                                            out_vec.insert(data.userdata.linearize());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
