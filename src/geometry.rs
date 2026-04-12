use crate::windmouse::Point;
use anyhow::{bail, Context, Result};
use rand::seq::IndexedRandom;
use scrap::{Capturer, Display};

#[derive(Debug)]
pub struct PixelColor {
    r: u8,
    g: u8,
    b: u8,
}

impl PixelColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub fn is_match(&self, target: &PixelColor, tolerance: u8) -> bool {
        (i16::from(self.r) - i16::from(target.r)).abs() <= i16::from(tolerance)
            && (i16::from(self.g) - i16::from(target.g)).abs() <= i16::from(tolerance)
            && (i16::from(self.b) - i16::from(target.b)).abs() <= i16::from(tolerance)
    }
}

/// Calculate minimum distance from a point to any edge of the polygon
fn min_distance_to_edges(point: &Point, polygon: &[Point]) -> f64 {
    let n = polygon.len();
    let mut min_dist = f64::MAX;

    for i in 0..n {
        let j = (i + 1) % n;
        let dist = point_to_line_segment_distance(point, &polygon[i], &polygon[j]);
        min_dist = min_dist.min(dist);
    }

    min_dist
}

/// Calculate distance from a point to a line segment
fn point_to_line_segment_distance(point: &Point, a: &Point, b: &Point) -> f64 {
    let px = f64::from(point.x);
    let py = f64::from(point.y);
    let ax = f64::from(a.x);
    let ay = f64::from(a.y);
    let bx = f64::from(b.x);
    let by = f64::from(b.y);
    let dx = bx - ax;
    let dy = by - ay;

    if dx == 0.0 && dy == 0.0 {
        // a and b are the same point
        return ((px - ax).powi(2) + (py - ay).powi(2)).sqrt();
    }

    // Calculate the parameter t that represents the projection of point onto the line
    let t = ((px - ax) * dx + (py - ay) * dy) / (dx * dx + dy * dy);
    let t = t.clamp(0.0, 1.0); // Clamp to segment

    // Calculate the closest point on the segment
    let closest_x = ax + t * dx;
    let closest_y = ay + t * dy;

    // Return distance to closest point
    ((px - closest_x).powi(2) + (py - closest_y).powi(2)).sqrt()
}

/// Ray casting algorithm for point-in-polygon test
fn point_in_polygon(point: &Point, polygon: &[Point]) -> bool {
    let mut inside = false;
    let n = polygon.len();

    for i in 0..n {
        let j = (i + 1) % n;
        let pi = &polygon[i];
        let pj = &polygon[j];

        // Check if the ray from point crosses this edge
        if ((pi.y > point.y) != (pj.y > point.y))
            && (point.x < (pj.x - pi.x) * (point.y - pi.y) / (pj.y - pi.y) + pi.x)
        {
            inside = !inside;
        }
    }

    inside
}

/// Graham scan algorithm to compute convex hull
fn convex_hull(points: &[Point]) -> Vec<Point> {
    if points.len() < 3 {
        return points.to_vec();
    }

    let mut sorted_points = points.to_vec();

    // Find the bottom-most point (or left-most if tie)
    let start_idx = sorted_points
        .iter()
        .enumerate()
        .min_by_key(|(_, p)| (p.y, p.x))
        .map(|(i, _)| i)
        .unwrap();

    sorted_points.swap(0, start_idx);
    let pivot = sorted_points[0];

    // Sort by polar angle with respect to pivot
    sorted_points[1..].sort_by(|a, b| {
        let det = cross_product(&pivot, a, b);
        if det == 0 {
            // Collinear points: closer point comes first
            let dist_a = distance_squared(&pivot, a);
            let dist_b = distance_squared(&pivot, b);
            dist_a.cmp(&dist_b)
        } else {
            det.cmp(&0).reverse()
        }
    });

    let mut hull = Vec::new();
    hull.push(sorted_points[0]);
    hull.push(sorted_points[1]);

    for &point in &sorted_points[2..] {
        while hull.len() > 1 {
            let b = hull[hull.len() - 1];
            let a = hull[hull.len() - 2];
            if cross_product(&a, &b, &point) <= 0 {
                hull.pop();
            } else {
                break;
            }
        }
        hull.push(point);
    }

    hull
}

fn cross_product(o: &Point, a: &Point, b: &Point) -> i32 {
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
}

fn distance_squared(a: &Point, b: &Point) -> i32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}

fn get_pixels_with_target_color(target_color: &PixelColor) -> Result<Vec<Point>> {
    // Get the primary display
    let display = Display::primary()?;
    let width = display.width();
    let mut capturer = Capturer::new(display).context("Failed to create Capturer object")?;
    let mut matches = Vec::new();
    const TOLERANCE: u8 = 3;

    loop {
        // Try to capture a frame
        if let Ok(frame) = capturer.frame() {
            // Iterate over the pixels
            for (i, bgra) in frame.chunks(4).enumerate() {
                let curr_color = PixelColor::new(bgra[2], bgra[1], bgra[0]);

                if curr_color.is_match(target_color, TOLERANCE) {
                    // Calculate pixel coordinates
                    let x = i % width;
                    let y = i / width;
                    matches.push(Point::new(i32::try_from(x)?, i32::try_from(y)?));
                }
            }
            break; // Exit after one frame
        }
    }
    Ok(matches)
}

pub fn find_color(target_color: &PixelColor) -> Result<Option<Point>> {
    let matches = get_pixels_with_target_color(target_color)?;
    let mut rng = rand::rng(); // Initialize RNG

    Ok(matches.choose(&mut rng).copied())
}

pub fn find_point_in_shape(target_color: &PixelColor) -> Result<Point> {
    let boundary_points = get_pixels_with_target_color(target_color)?;

    if boundary_points.is_empty() {
        bail!("No pixels found matching the target color");
    }

    if boundary_points.len() == 1 {
        bail!("Only one pixel found matching the target color, cannot determine shape");
    }

    // Find bounding box
    let min_x = boundary_points.iter().map(|p| p.x).min().unwrap();
    let max_x = boundary_points.iter().map(|p| p.x).max().unwrap();
    let min_y = boundary_points.iter().map(|p| p.y).min().unwrap();
    let max_y = boundary_points.iter().map(|p| p.y).max().unwrap();

    // Create convex hull from boundary points to form a proper polygon
    let polygon = convex_hull(&boundary_points);

    if polygon.len() < 3 {
        // Not enough points to form a polygon
        bail!("Not enough distinct pixels to form a polygon");
    }

    // Try to find a random point inside the polygon, biased away from edges
    const MAX_ATTEMPTS: u32 = 1000;
    const MIN_EDGE_DISTANCE: f64 = 10.0; // Minimum distance from any edge

    let mut best_candidate: Option<Point> = None;
    let mut best_distance = 0.0;

    for _ in 0..MAX_ATTEMPTS {
        let candidate = Point::new(
            rand::random_range(min_x..=max_x),
            rand::random_range(min_y..=max_y),
        );

        if point_in_polygon(&candidate, &polygon) {
            let dist = min_distance_to_edges(&candidate, &polygon);

            // If we found a point with good distance from edges, return it
            if dist >= MIN_EDGE_DISTANCE {
                return Ok(candidate);
            }

            // Keep track of the best candidate (furthest from edges)
            if dist > best_distance {
                best_distance = dist;
                best_candidate = Some(candidate);
            }
        }
    }

    if let Some(candidate) = best_candidate {
        Ok(candidate)
    } else {
        bail!(
            "Failed to find a point inside the shape after {} attempts",
            MAX_ATTEMPTS
        );
    }
}
