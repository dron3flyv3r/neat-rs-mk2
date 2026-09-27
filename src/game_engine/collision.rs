use std::array;

use sola_raylib::prelude::*;

#[derive(Debug, Clone)]
pub struct CollisionRect {
    // Center of the rectangle
    pub position: Vector2,
    pub rotation: f32,
    pub size: Vector2,

    local_points: [Vector2; 4],
    normals: [Vector2; 2],
}

#[derive(Debug, Clone)]
pub struct CollisionCircle {
    pub position: Vector2,
    pub radius: f32,
}

#[derive(Debug, Clone)]
pub enum CollisionShape {
    Rectangle(CollisionRect),
    Circle(CollisionCircle),
}

pub struct CollisionResult {
    pub correction_vector: Vector2,
}

impl CollisionRect {
    pub fn new(position: Vector2, rotation: f32, size: Vector2) -> Self {
        let mut object = Self {
            position,
            size,
            rotation,
            local_points: array::from_fn(|_| Vector2 { x: 0.0, y: 0.0 }),
            normals: array::from_fn(|_| Vector2 { x: 0.0, y: 0.0 }),
        };

        object.update_local_points();
        object.update_normals();
        object
    }

    pub fn update_position(&mut self, position: Vector2) {
        if position == self.position {
            return;
        }
        self.position = position;
    }

    pub fn update_rotation(&mut self, rotation: f32) {
        if rotation == self.rotation {
            return;
        }
        self.rotation = rotation;
        self.update_normals();
    }

    pub fn update_size(&mut self, size: Vector2) {
        if size == self.size {
            return;
        }
        self.size = size;
        self.update_local_points();
    }

    fn update_local_points(&mut self) {
        let hx = self.size.x / 2.0;
        let hy = self.size.y / 2.0;
        self.local_points = [
            Vector2 { x: -hx, y: hy },
            Vector2 { x: hx, y: hy },
            Vector2 { x: hx, y: -hy },
            Vector2 { x: -hx, y: -hy },
        ];
    }

    pub fn get_world_points(&self) -> [Vector2; 4] {
        self.local_points
            .map(|p| p.rotated(self.rotation) + self.position)
    }

    pub fn get_local_points(&self) -> [Vector2; 4] {
        self.local_points
    }

    fn update_normals(&mut self) {
        self.normals = [
            Vector2 { x: 0.0, y: 1.0 }.rotated(self.rotation),
            Vector2 { x: -1.0, y: 0.0 }.rotated(self.rotation),
        ];
    }

    pub fn get_normals(&self) -> [Vector2; 2] {
        self.normals
    }
}

impl CollisionCircle {
    pub fn new(position: Vector2, radius: f32) -> Self {
        Self { position, radius }
    }

    pub fn update_position(&mut self, position: Vector2) {
        if position == self.position {
            return;
        }
        self.position = position;
    }

    pub fn update_radius(&mut self, radius: f32) {
        if radius == self.radius {
            return;
        }
        self.radius = radius;
    }
}

pub trait Collision {
    fn collision_shape(&mut self) -> CollisionShape;

    fn collision_enabled(&self) -> bool;

    fn is_colliding<T: Collision>(&mut self, other: &mut T) -> Option<CollisionResult> {
        if !self.collision_enabled() || !other.collision_enabled() {
            return None;
        }

        self.collision_shape()
            .is_colliding(&other.collision_shape())
    }

    fn check_collision_at_position<T: Collision>(
        &mut self,
        other: &mut T,
        position: &Vector2,
    ) -> Option<CollisionResult> {
        if !self.collision_enabled() || !other.collision_enabled() {
            return None;
        }

        let mut self_shape = self.collision_shape();
        self_shape.with_position(position);

        if let Some(mut collision_result) = self_shape.is_colliding(&other.collision_shape()) {
            if collision_result.correction_vector.x.abs() < 1.0 {
                collision_result.correction_vector.x = 0.0;
            }
            if collision_result.correction_vector.y.abs() < 1.0 {
                collision_result.correction_vector.y = 0.0;
            }
            Some(collision_result)
        } else {
            None
        }
    }
}

impl CollisionShape {
    pub fn with_position(&mut self, position: &Vector2) {
        match self {
            CollisionShape::Rectangle(rect) => rect.position = *position,
            CollisionShape::Circle(circle) => circle.position = *position,
        }
    }

    pub fn is_colliding(&self, other: &CollisionShape) -> Option<CollisionResult> {
        match (self, other) {
            (CollisionShape::Rectangle(rect1), CollisionShape::Rectangle(rect2)) => {
                Self::rect_vs_rect(rect1, rect2)
            }

            (CollisionShape::Rectangle(rect1), CollisionShape::Circle(circle2)) => {
                Self::rect_vs_circle(rect1, circle2)
            }

            (CollisionShape::Circle(circle1), CollisionShape::Rectangle(rect2)) => {
                Self::rect_vs_circle(rect2, circle1)
            }

            (CollisionShape::Circle(circle1), CollisionShape::Circle(circle2)) => {
                Self::circle_vs_circle(circle1, circle2)
            }
        }
    }

    fn project_points_onto_vector(points: &[Vector2], vector: &Vector2) -> (f32, f32) {
        let mut min = f32::MAX;
        let mut max = f32::MIN;

        for point in points {
            let projection = point.x * vector.x + point.y * vector.y;

            min = min.min(projection);
            max = max.max(projection);
        }

        (min, max)
    }

    fn rect_vs_rect(rect1: &CollisionRect, rect2: &CollisionRect) -> Option<CollisionResult> {
        let point1 = rect1.get_world_points();
        let point2 = rect2.get_world_points();
        let normals1 = rect1.get_normals();
        let normals2 = rect2.get_normals();

        let mut smallest_overlap = f32::MAX;
        let mut smallest_normal = Vector2 { x: 0.0, y: 0.0 };

        for normal in normals1.iter().chain(normals2.iter()) {
            let (rect1_min, rect1_max) = Self::project_points_onto_vector(&point1, normal);
            let (rect2_min, rect2_max) = Self::project_points_onto_vector(&point2, normal);

            if rect1_max < rect2_min || rect2_max < rect1_min {
                return None;
            }

            let overlap = rect1_max.min(rect2_max) - rect1_min.max(rect2_min);

            if overlap < smallest_overlap {
                smallest_overlap = overlap;
                smallest_normal = *normal;
            }
        }

        let direction = rect1.position - rect2.position;

        if direction.dot(smallest_normal) < 0.001 {
            smallest_normal = -smallest_normal;
        }

        Some(CollisionResult {
            correction_vector: smallest_normal * smallest_overlap,
        })
    }

    fn rect_vs_circle(rect: &CollisionRect, circle: &CollisionCircle) -> Option<CollisionResult> {
        let (hx, hy) = (rect.size.x / 2.0, rect.size.y / 2.0);

        let closest_x = circle
            .position
            .x
            .clamp(rect.position.x - hx, rect.position.x + hx);
        let closest_y = circle
            .position
            .y
            .clamp(rect.position.y - hy, rect.position.y + hy);

        let dx = circle.position.x - closest_x;
        let dy = circle.position.y - closest_y;
        let dist_sq = dx * dx + dy * dy;

        if dist_sq > circle.radius * circle.radius {
            return None;
        }

        let dist = dist_sq.sqrt();

        if dist == 0.0 {
            // circle center is inside rect
            let offset = Vector2 {
                x: circle.position.x - rect.position.x,
                y: circle.position.y - rect.position.y,
            };

            let px = hx - offset.x.abs();
            let py = hy - offset.y.abs();

            return Some(CollisionResult {
                correction_vector: {
                    Vector2 {
                        x: if offset.x >= 0.0 { px } else { -px },
                        y: if offset.y >= 0.0 { py } else { -py },
                    }
                },
            });
        }

        let normal = Vector2 {
            x: dx / dist,
            y: dy / dist,
        };

        let penetration = circle.radius - dist;

        Some(CollisionResult {
            correction_vector: Vector2 {
                x: normal.x * penetration,
                y: normal.y * penetration,
            },
        })
    }
    fn circle_vs_circle(
        circle1: &CollisionCircle,
        circle2: &CollisionCircle,
    ) -> Option<CollisionResult> {
        let dx = circle2.position.x - circle1.position.x;
        let dy = circle2.position.y - circle1.position.y;
        let dist_sq = dx * dx + dy * dy;
        let radius_sum = circle1.radius + circle2.radius;

        if dist_sq >= radius_sum * radius_sum {
            return None;
        }

        let dist = dist_sq.sqrt().max(0.0001);
        let normal = Vector2 {
            x: dx / dist,
            y: dy / dist,
        };

        let penetration = radius_sum - dist;

        Some(CollisionResult {
            correction_vector: Vector2 {
                x: normal.x * penetration,
                y: normal.y * penetration,
            },
        })
    }
}
