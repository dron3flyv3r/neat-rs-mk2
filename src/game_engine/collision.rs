use std::array;

use sola_raylib::prelude::*;

#[derive(Debug, Clone)]
pub struct CollisionPolygon {
    pub position: Vector2,
    pub rotation: f32,
    pub local_points: Vec<Vector2>,

    normals: Vec<Vector2>,
}

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
    Polygon(CollisionPolygon),
    Rectangle(CollisionRect),
    Circle(CollisionCircle),
}

pub struct CollisionResult {
    // Correction from self to other, so that self is no longer colliding with other
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

impl CollisionPolygon {
    pub fn new(position: Vector2, rotation: f32, local_points: Vec<Vector2>) -> Self {
        let mut object = Self {
            position,
            rotation,
            local_points,
            normals: Vec::new(),
        };
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

    pub fn update_local_points(&mut self, local_points: Vec<Vector2>) {
        if local_points == self.local_points {
            return;
        }
        self.local_points = local_points;
    }

    pub fn get_world_points(&self) -> Vec<Vector2> {
        self.local_points
            .iter()
            .map(|p| p.rotated(self.rotation) + self.position)
            .collect()
    }

    fn update_normals(&mut self) {
        let mut normals = Vec::new();
        let len = self.local_points.len();
        for i in 0..len {
            let p1 = self.local_points[i];
            let p2 = self.local_points[(i + 1) % len];
            let edge = p2 - p1;
            let normal = Vector2 {
                x: -edge.y,
                y: edge.x,
            }
            .normalized()
            .rotated(self.rotation);
            normals.push(normal);
        }
        self.normals = normals;
    }

    fn get_normals(&self) -> &Vec<Vector2> {
        &self.normals
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
            CollisionShape::Polygon(polygon) => polygon.position = *position,
        }
    }

    pub fn is_colliding(&self, other: &CollisionShape) -> Option<CollisionResult> {
        match (self, other) {
            (CollisionShape::Polygon(poly1), CollisionShape::Polygon(poly2)) => {
                Self::poly_vs_poly(self, poly1, poly2)
            }
            (CollisionShape::Polygon(poly1), CollisionShape::Rectangle(rect2)) => {
                Self::poly_vs_rect(self, poly1, rect2)
            }
            (CollisionShape::Rectangle(rect1), CollisionShape::Polygon(poly2)) => {
                Self::poly_vs_rect(self, poly2, rect1)
            }
            (CollisionShape::Polygon(poly1), CollisionShape::Circle(circle2)) => {
                Self::poly_vs_circle(poly1, circle2)
            }
            (CollisionShape::Circle(circ1), CollisionShape::Polygon(poly2)) => {
                Self::poly_vs_circle(poly2, circ1)
            }
            (CollisionShape::Rectangle(rect1), CollisionShape::Rectangle(rect2)) => {
                Self::rect_vs_rect(self, rect1, rect2)
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

    fn run_sat_collision(
        &self,
        points1: &[Vector2],
        points2: &[Vector2],
        normals1: &[Vector2],
        normals2: &[Vector2],
        center1: &Vector2,
        center2: &Vector2,
    ) -> Option<CollisionResult> {
        let mut smallest_overlap = f32::MAX;
        let mut smallest_normal = Vector2 { x: 0.0, y: 0.0 };

        for normal in normals1.iter().chain(normals2.iter()) {
            let (poly1_min, poly1_max) = Self::project_points_onto_vector(points1, normal);
            let (poly2_min, poly2_max) = Self::project_points_onto_vector(points2, normal);

            if poly1_max < poly2_min || poly2_max < poly1_min {
                return None;
            }

            let overlap = poly1_max.min(poly2_max) - poly1_min.max(poly2_min);

            if overlap < smallest_overlap {
                smallest_overlap = overlap;
                smallest_normal = *normal;
            }
        }

        let delta = *center2 - *center1;

        let dot = delta.x * smallest_normal.x + delta.y * smallest_normal.y;

        if dot < 0.0 {
            smallest_normal = -smallest_normal;
        }

        Some(CollisionResult {
            correction_vector: -smallest_normal * smallest_overlap,
        })
    }

    fn poly_vs_poly(
        &self,
        poly1: &CollisionPolygon,
        poly2: &CollisionPolygon,
    ) -> Option<CollisionResult> {
        let points1 = &poly1.get_world_points();
        let points2 = &poly2.get_world_points();
        let normals1 = &poly1.normals;
        let normals2 = &poly2.normals;

        self.run_sat_collision(
            points1,
            points2,
            normals1,
            normals2,
            &poly1.position,
            &poly2.position,
        )
    }

    fn poly_vs_rect(
        &self,
        poly: &CollisionPolygon,
        rect: &CollisionRect,
    ) -> Option<CollisionResult> {
        let points1 = &poly.get_world_points();
        let points2 = &rect.get_world_points();
        let normals1 = &poly.normals;
        let normals2 = &rect.normals;

        self.run_sat_collision(
            points1,
            points2,
            normals1,
            normals2,
            &poly.position,
            &rect.position,
        )
    }

    fn poly_vs_circle(
        poly: &CollisionPolygon,
        circle: &CollisionCircle,
    ) -> Option<CollisionResult> {
        let points = &poly.get_world_points();
        let normals = poly.get_normals();

        let mut smallest_overlap = f32::MAX;
        let mut smallest_normal = Vector2 { x: 0.0, y: 0.0 };

        for normal in normals.iter() {
            let (poly_min, poly_max) = Self::project_points_onto_vector(points, normal);
            let circle_projection = circle.position.x * normal.x + circle.position.y * normal.y;
            let (circle_min, circle_max) = (
                circle_projection - circle.radius,
                circle_projection + circle.radius,
            );

            if poly_max < circle_min || circle_max < poly_min {
                return None;
            }

            let overlap = poly_max.min(circle_max) - poly_min.max(circle_min);

            if overlap < smallest_overlap {
                smallest_overlap = overlap;
                smallest_normal = *normal;
            }
        }

        let delta = poly.position - circle.position;

        let dot = delta.x * smallest_normal.x + delta.y * smallest_normal.y;

        if dot < 0.0 {
            smallest_normal = -smallest_normal;
        }

        Some(CollisionResult {
            correction_vector: -smallest_normal * smallest_overlap,
        })
    }

    fn rect_vs_rect(
        &self,
        rect1: &CollisionRect,
        rect2: &CollisionRect,
    ) -> Option<CollisionResult> {
        let point1 = &rect1.get_world_points();
        let point2 = &rect2.get_world_points();
        let normals1 = &rect1.get_normals();
        let normals2 = &rect2.get_normals();

        self.run_sat_collision(
            point1,
            point2,
            normals1,
            normals2,
            &rect1.position,
            &rect2.position,
        )
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
                        x: if offset.x >= 0.0 { -px } else { px },
                        y: if offset.y >= 0.0 { -py } else { py },
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
                x: -normal.x * penetration,
                y: -normal.y * penetration,
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
                x: -normal.x * penetration,
                y: -normal.y * penetration,
            },
        })
    }
}
