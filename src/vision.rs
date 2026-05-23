//! Geometric algorithms and color-based pixel detection for screen automation.
//!
//! This module provides functions for finding pixels by color, computing convex hulls,
//! point-in-polygon tests, and selecting points within colored shapes with edge distance bias.
use crate::windmouse::Point;

use anyhow::{bail, Context, Result};
use image::ImageReader;
use image::{GrayImage, ImageBuffer, Rgba};
use imageproc::template_matching::{find_extremes, MatchTemplateMethod};
use rand::seq::IndexedRandom;
use scrap::{Capturer, Display};
use std::path::Path;

/// RGB color representation for pixel matching
#[derive(Debug)]
pub struct PixelColor {
    r: u8,
    g: u8,
    b: u8,
}

impl PixelColor {
    /// Creates a new PixelColor with the given RGB values.
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Checks if this color matches the target color within the given tolerance.
    pub fn is_match(&self, target: &PixelColor, tolerance: u8) -> bool {
        (i16::from(self.r) - i16::from(target.r)).abs() <= i16::from(tolerance)
            && (i16::from(self.g) - i16::from(target.g)).abs() <= i16::from(tolerance)
            && (i16::from(self.b) - i16::from(target.b)).abs() <= i16::from(tolerance)
    }
}

/// Calculates minimum distance from a point to any edge of the polygon.
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

/// Calculates distance from a point to a line segment.
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

/// Determines if a point is inside a polygon using ray casting algorithm.
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

/// Computes the convex hull of a set of points using
/// [Graham scan](https://en.wikipedia.org/wiki/Graham_scan) algorithm.
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

/// Calculates the cross product of vectors OA and OB.
fn cross_product(o: &Point, a: &Point, b: &Point) -> i32 {
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x)
}

/// Calculates the squared Euclidean distance between two points.
fn distance_squared(a: &Point, b: &Point) -> i32 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    dx * dx + dy * dy
}

/// Captures screen and returns all pixels matching the target color within tolerance.
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

/// Finds and returns a random pixel position matching the target color, if any exist.
pub fn find_color(target_color: &PixelColor) -> Result<Option<Point>> {
    let matches = get_pixels_with_target_color(target_color)?;
    let mut rng = rand::rng();

    Ok(matches.choose(&mut rng).copied())
}

/// Finds a point inside the shape formed by pixels matching the target color, biased away from edges.
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

/// Captures the primary display and returns it as a grayscale image.
fn capture_screen() -> Result<GrayImage> {
    // Initialize the display capturer for the primary monitor
    let display =
        Display::primary().context("Failed to identify or access the primary display monitor")?;

    let mut capturer = Capturer::new(display).context(
        "Failed to initialize system capture session. Check OS screen recording permissions.",
    )?;

    let width = capturer.width();
    let height = capturer.height();

    // Loop until a valid display frame is ready
    let frame_buffer = loop {
        if let Ok(frame) = capturer.frame() {
            break frame.to_vec();
        }
    };

    // Convert raw scrap buffer from BGRA to RGBA channels
    let mut rgba_raw = Vec::with_capacity(frame_buffer.len());
    for chunk in frame_buffer.chunks_exact(4) {
        rgba_raw.push(chunk[2]); // R
        rgba_raw.push(chunk[1]); // G
        rgba_raw.push(chunk[0]); // B
        rgba_raw.push(chunk[3]); // A
    }

    // Wrap raw byte buffer into an ImageBuffer container
    let src_rgba: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(width as u32, height as u32, rgba_raw).context(
            "Captured screen byte buffer dimensions did not match required resolution constraints",
        )?;

    // Convert to grayscale for template matching
    Ok(image::DynamicImage::ImageRgba8(src_rgba).to_luma8())
}

/// Generates a random point within the rectangle defined by the origin and dimensions.
fn get_rand_point_in_rect(origin: Point, width: u32, height: u32) -> Result<Point> {
    // Generate random offsets within 0.0 (inclusive) and dimension (exclusive)
    let random_x = i32::try_from(rand::random_range(0..width))?;
    let random_y = i32::try_from(rand::random_range(0..height))?;

    // Calculate the absolute coordinates of the point
    let point = Point {
        x: origin.x + random_x,
        y: origin.y + random_y,
    };

    Ok(point)
}

/// Finds the location of the target image on the screen using template matching.
pub fn find_image_on_screen(target_image: &Path) -> Result<Point> {
    // Capture the screen using our separate function
    let src = capture_screen().context("Could not extract a valid desktop screenshot frame")?;

    // Load template image converted to grayscale
    let temp_dynamic = ImageReader::open(target_image)
        .context("Failed to locate or open 'template.png' asset from disk")?
        .decode()
        .context("Failed to parse and decode target 'template.png' format structure")?;
    let temp = temp_dynamic.to_luma8();

    // Run template matching
    let result_image = imageproc::template_matching::match_template_parallel(
        &src,
        &temp,
        MatchTemplateMethod::SumOfSquaredErrors,
    );

    // Find the location of the best match
    let extremes = find_extremes(&result_image);
    let best_match_pos = extremes.min_value_location;
    let confidence_score = extremes.min_value;

    // Dynamically calculate a confidence threshold based on template size and a variance factor
    const MAX_PIXEL_VARIANCE: f32 = 15.0;
    let temp_width = temp.width();
    let temp_height = temp.height();
    let total_pixels = (temp_width * temp_height) as f32;
    let dynamic_threshold = total_pixels * MAX_PIXEL_VARIANCE.powi(2);

    // Check if the confidence score is below the dynamic threshold to determine if a valid match
    // was found
    if confidence_score <= dynamic_threshold {
        // Return a random point within the matched region to avoid clicking the exact same pixel
        // every time
        let origin = Point::new(
            i32::try_from(best_match_pos.0)?,
            i32::try_from(best_match_pos.1)?,
        );
        Ok(get_rand_point_in_rect(origin, temp_width, temp_height)?)
    } else {
        bail!(
            "No match found for template image. Best match confidence score {} exceeded threshold {}",
            confidence_score,
            dynamic_threshold
        );
    }
}
