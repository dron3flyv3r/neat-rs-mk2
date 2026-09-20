use sola_raylib::prelude::*;

#[derive(Debug)]
pub struct CollisionRect {
    // The position of the top-left corner of the rectangle
    pub position: Vector2,
    pub size: Vector2,
}

#[derive(Debug)]
pub struct CollisionCircle {
    pub position: Vector2,
    pub radius: f32,
}

#[derive(Debug)]
pub enum CollisionShape {
    Rectangle(CollisionRect),
    Circle(CollisionCircle),
}

pub struct CollisionResult {
    pub correction_vector: Vector2,
}

pub trait Collision {
    fn collision_shape(&self) -> CollisionShape;

    fn collision_enabled(&self) -> bool;

    fn is_colliding<T: Collision>(&self, other: &T) -> Option<CollisionResult> {
        if !self.collision_enabled() || !other.collision_enabled() {
            return None;
        }

        self.collision_shape()
            .is_colliding(&other.collision_shape())
    }

    fn check_collision_at_position<T: Collision>(
        &self,
        other: &T,
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

    fn rect_vs_rect(rect1: &CollisionRect, rect2: &CollisionRect) -> Option<CollisionResult> {
        let rect1_right = rect1.position.x + rect1.size.x;
        let rect1_bottom = rect1.position.y + rect1.size.y;

        let rect2_right = rect2.position.x + rect2.size.x;
        let rect2_bottom = rect2.position.y + rect2.size.y;

        let overlap_x =
            (rect1_right.min(rect2_right) - rect1.position.x.max(rect2.position.x)).max(0.0);
        let overlap_y =
            (rect1_bottom.min(rect2_bottom) - rect1.position.y.max(rect2.position.y)).max(0.0);

        if overlap_x <= 0.0 || overlap_y <= 0.0 {
            return None;
        }

        let c1 = Vector2 {
            x: rect1.position.x + rect1.size.x / 2.0,
            y: rect1.position.y + rect1.size.y / 2.0,
        };

        let c2 = Vector2 {
            x: rect2.position.x + rect2.size.x / 2.0,
            y: rect2.position.y + rect2.size.y / 2.0,
        };

        if overlap_x < overlap_y {
            let dir = if c1.x < c2.x { 1.0 } else { -1.0 };
            Some(CollisionResult {
                correction_vector: Vector2 {
                    x: dir * overlap_x,
                    y: 0.0,
                },
            })
        } else {
            let dir = if c1.y < c2.y { 1.0 } else { -1.0 };
            Some(CollisionResult {
                correction_vector: Vector2 {
                    x: 0.0,
                    y: dir * overlap_y,
                },
            })
        }
    }

    fn rect_vs_circle(rect: &CollisionRect, circle: &CollisionCircle) -> Option<CollisionResult> {
        let closest_x = circle
            .position
            .x
            .clamp(rect.position.x, rect.position.x + rect.size.x);
        let closest_y = circle
            .position
            .y
            .clamp(rect.position.y, rect.position.y + rect.size.y);

        let dx = circle.position.x - closest_x;
        let dy = circle.position.y - closest_y;
        let dist_sq = dx * dx + dy * dy;

        if dist_sq > circle.radius * circle.radius {
            return None;
        }

        let dist = dist_sq.sqrt();

        if dist == 0.0 {
            // circle center is inside rect
            let half_w = rect.size.x / 2.0;
            let half_h = rect.size.y / 2.0;
            let center = Vector2 {
                x: rect.position.x + half_w,
                y: rect.position.y + half_h,
            };

            let offset = Vector2 {
                x: circle.position.x - center.x,
                y: circle.position.y - center.y,
            };

            let px = half_w - offset.x.abs();
            let py = half_h - offset.y.abs();

            if px < py {
                let dir = if offset.x >= 0.0 { 1.0 } else { -1.0 };
                return Some(CollisionResult {
                    correction_vector: Vector2 {
                        x: dir * (circle.radius + px),
                        y: 0.0,
                    },
                });
            } else {
                let dir = if offset.y >= 0.0 { 1.0 } else { -1.0 };
                return Some(CollisionResult {
                    correction_vector: Vector2 {
                        x: 0.0,
                        y: dir * (circle.radius + py),
                    },
                });
            }
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
