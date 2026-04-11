use anyhow::{Context, Result};
use enigo::{Coordinate, Enigo, Mouse, Settings};
use std::fmt::Display;
use std::time::Duration;

/// Starting and ending coordinates for mouse movement
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

impl Point {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    fn distance_to(&self, other: &Point) -> f64 {
        f64::hypot(f64::from(other.x - self.x), f64::from(other.y - self.y))
    }
}

impl Display for Point {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:.1}, {:.1})", self.x, self.y)
    }
}

/// Parameters controlling the wind mouse algorithm behavior
#[derive(Debug, Clone, Copy)]
pub struct WindMouseParams {
    /// Gravity strength - pulls towards destination
    pub gravity: f64,
    /// Wind strength - adds randomness
    pub wind: f64,
    /// Max velocity
    pub max_velocity: f64,
    /// Distance threshold for wind behavior change
    pub distance_threshold: f64,
}

impl WindMouseParams {
    fn new() -> Self {
        // Constants used here were taken from DreamBot source code
        // https://dreambot.org/forums/index.php?/topic/21147-windmouse-custom-mouse-movement-algorithm/
        let gravity = rand::random_range(4.0..20.0);
        let wind = rand::random_range(1.0..10.0);
        let max_velocity = rand::random_range(15.0 / 2.0..15.0);
        let distance_threshold = rand::random_range(5.0..25.0);

        Self {
            gravity,
            wind,
            max_velocity,
            distance_threshold,
        }
    }
}

/// Wind mouse controller that owns the Enigo instance
pub struct WindMouse {
    enigo: Enigo,
}

impl WindMouse {
    const MOUSE_POLL_INTERVAL_MS: Duration = Duration::from_millis(8);

    /// Moves mouse cursor from start to destination using wind mouse algorithm
    fn wind_mouse(&mut self, start: Point, dest: Point, params: WindMouseParams) -> Result<()> {
        let sqrt3 = 3.0_f64.sqrt();
        let sqrt5 = 5.0_f64.sqrt();

        let mut current = start;
        let mut velocity = (0.0, 0.0);
        let mut wind = (0.0, 0.0);
        let mut max_velocity = params.max_velocity;

        loop {
            let distance = current.distance_to(&dest);

            if distance < 1.0 {
                break;
            }

            let wind_magnitude = params.wind.min(distance);

            if distance >= params.distance_threshold {
                // Add wind randomness
                wind.0 =
                    wind.0 / sqrt3 + (2.0 * rand::random::<f64>() - 1.0) * wind_magnitude / sqrt5;
                wind.1 =
                    wind.1 / sqrt3 + (2.0 * rand::random::<f64>() - 1.0) * wind_magnitude / sqrt5;
            } else {
                // Reduce wind as we approach target
                wind.0 /= sqrt3;
                wind.1 /= sqrt3;

                // Adjust max velocity near target
                max_velocity = if max_velocity < 3.0 {
                    rand::random::<f64>() * 3.0 + 3.0
                } else {
                    max_velocity / sqrt5
                };
            }

            // Calculate gravity pull towards destination
            let gravity_pull = (
                params.gravity * f64::from(dest.x - current.x) / distance,
                params.gravity * f64::from(dest.y - current.y) / distance,
            );

            // Update velocity
            velocity.0 += wind.0 + gravity_pull.0;
            velocity.1 += wind.1 + gravity_pull.1;

            // Clip velocity to max
            let velocity_magnitude = f64::hypot(velocity.0, velocity.1);
            if velocity_magnitude > max_velocity {
                let clip = max_velocity / 2.0 + rand::random::<f64>() * max_velocity / 2.0;
                let scale = clip / velocity_magnitude;
                velocity.0 *= scale;
                velocity.1 *= scale;
            }

            // Update position
            let next = Point::new(
                current.x + velocity.0.round() as i32,
                current.y + velocity.1.round() as i32,
            );

            if next.x != current.x || next.y != current.y {
                current = next;

                // Apply the mouse poll interval to control update frequency
                std::thread::sleep(WindMouse::MOUSE_POLL_INTERVAL_MS);
                self.enigo
                    .move_mouse(current.x, current.y, Coordinate::Abs)?;
            }
        }

        Ok(())
    }

    pub fn new() -> Result<Self> {
        let enigo = Enigo::new(&Settings::default()).context("failed to init enigo")?;

        Ok(Self { enigo })
    }

    pub fn move_to(&mut self, start: Point, dest: Point) -> Result<()> {
        let params = WindMouseParams::new();

        self.wind_mouse(start, dest, params)
            .context(format!("failed to move to destination {}", dest))?;

        Ok(())
    }
}
