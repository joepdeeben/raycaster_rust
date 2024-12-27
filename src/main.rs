use std::io;
use speedy2d;
use ndarray::{array, range, Array, Ix2};
use speedy2d::color::Color;
use speedy2d::shape::Rectangle;
use speedy2d::window::{KeyScancode, VirtualKeyCode};
use speedy2d::{Graphics2D, Window};
use speedy2d::window::{WindowHandler, WindowHelper};
use rayon::prelude::*;

/// Builds a static 6x6 world array (walls = 1, empty = 0)
fn draw_world() -> [[i32; 6]; 6] {
    [
        [1, 1, 1, 1, 1, 1],
        [1, 0, 0, 0, 0, 1],
        [1, 0, 0, 0, 0, 1],
        [1, 0, 0, 0, 0, 1],
        [1, 0, 0, 0, 0, 1],
        [1, 1, 1, 1, 1, 1],
    ]
}

/// Returns true if the cell in world at (x, y-1) is non-empty (a wall)
#[inline(always)]
fn check_collision(world: &[[i32; 6]; 6], x: i32, y: i32) -> bool {
    // Adjusted bounds: y should be at least 1 to prevent underflow when doing y - 1
    if x < 0 || x >= 6 || y < 1 || y > 6 {
        return false;
    }
    // Direct indexing is faster than using `get` and `and_then`
    world[(y - 1) as usize][x as usize] != 0
}

/// “Casts” upwards (along y) and returns a distance, if collision is found
fn check_y(world: [[i32; 6]; 6], pov: f32, px: f32, py: f32, tan_pov: f32) -> f32 {
    // set ray origin to top of current tile
    let floor_boundary_y = py.ceil();

    // calculate distance by subtracting player y from top of current tile
    let y_collision_dist = floor_boundary_y - py;

    // calculate x at top of current tile by toa + player x
    let mut x_collision = px + (y_collision_dist / tan_pov);

    // set y collision to top of current y cell
    let mut y_col = py.floor() as i32;

    // calculate slope for stepping in the x direction
    let xa = if tan_pov == 0.0 { 0.0 } else { 1.0 / tan_pov };
    let mut dist = 0.0;

    // Step up the grid from py downward
    for _ in 0..6 {
        y_col -= 1;
        x_collision += xa;

        // If a collision is found, compute distance to that point
        if check_collision(&world, x_collision.floor() as i32, y_col) {
            let dy = py - (y_col as f32);
            let dx = x_collision - px;
            dist += (dx * dx + dy * dy).sqrt();
            return dist;
        }
    }
    return 1000.0;
}

fn check_x(world: [[i32; 6]; 6], pov: f32, px: f32, py: f32, tan_pov: f32) -> f32 {
    // set ray origin to right border of current tile
    let floor_boundary_x = px.floor();
    // calculate distance to right border by subtracting px from right border
    let x_collision_dist = floor_boundary_x - px;
    // calculate y at right of current tile by toa + player x
    let mut y_collision = py - (x_collision_dist * tan_pov)  ;

    let mut x_col = px.ceil() as i32;

    // Slope for stepping in the y direction
    let ya = if tan_pov == 0.0 { 0.0 } else { tan_pov };
    let mut dist = 0.0;

    // Step up the grid from py downward
    for _ in 0..6{
        x_col += 1;
        y_collision -= ya;

        // If a collision is found, compute distance to that point
        if check_collision(&world, x_col, y_collision.ceil() as i32) {
            let dx = (x_col as f32)- px;
            let dy = y_collision - py;
            dist += (dx * dx + dy * dy).sqrt();
            return dist;
        }
    }
    return 1000.0;
}


/// Wrapper to cast a ray in the y direction, returning 0.0 if no collision
fn cast_ray(world: [[i32; 6]; 6], pov: f32, px: f32, py: f32) -> f32 {
    let tan_pov = pov.tan();
    let distance_y = check_y(world, pov, px, py, tan_pov);
    let distance_x = check_x(world, pov, px, py, tan_pov);
    if distance_y > distance_x { distance_x } else {distance_y}
}

/// Stores data needed by our window: world, POV, and rectangle bounds
struct MyWindowHandler {
    world: [[i32; 6]; 6],
    pov: f32,
    rect_height_top: f32,
    rect_height_bot: f32,
    fov: f32,
    player_x: f32,
    player_y: f32
}

impl WindowHandler for MyWindowHandler {
    fn on_key_down(
        &mut self,
        helper: &mut WindowHelper<()>,
        key_code: Option<VirtualKeyCode>,
        _scancode: KeyScancode
    ) {
        match key_code {
            Some(VirtualKeyCode::Left)  => self.pov += 1.0_f32.to_radians(),
            Some(VirtualKeyCode::Right) => self.pov -= 1.0_f32.to_radians(),
            Some(VirtualKeyCode::Up)  => self.player_y -= 0.1_f32,
            _ => {}
        }
        if self.pov.to_degrees() > 360.0 { self.pov -= 360.0_f32.to_radians(); }
        if self.pov.to_degrees() < -360.0 { self.pov += 360.0_f32.to_radians(); }


        helper.request_redraw();
    }

    fn on_draw(
        &mut self,
        helper: &mut WindowHelper<()>,
        graphics: &mut Graphics2D

    ) {
        graphics.clear_screen(Color::from_rgb(0.8, 0.9, 1.0));

        // calculate angle between individual rays as seen from player
        let half_fov = self.fov / 2.0;
        let screen_mid_y = 240.0;
        let screen_width = 640;
        let angle_between_rays = self.fov / screen_width as f32;


        // Parallel computation of rectangles
        let rectangles: Vec<Rectangle> = (0..screen_width).into_par_iter()
            .map(|i| {
                let i_f32 = i as f32;
                let ray_angle = self.pov + half_fov - i_f32 * angle_between_rays;
                let len = cast_ray(self.world, ray_angle, self.player_x, self.player_y).max(0.001); // Prevent division by zero
                let rect_height_top = screen_mid_y - (screen_mid_y / len);
                let rect_height_bot = screen_mid_y + (screen_mid_y / len);
                Rectangle::from_tuples(
                    (i_f32, rect_height_top),
                    (i_f32 + 1.0, rect_height_bot),
                )
            })
            .collect();

        // Sequentially draw each rectangle
        for rect in &rectangles {
            graphics.draw_rectangle(rect, Color::RED);
        }

        helper.request_redraw();
    }
}

fn main() {
    // Build the world
    let world = draw_world();

    // Initial POV and distance
    let pov = 90_f32.to_radians();
    let len = cast_ray(world, pov, 3.0, 3.0);
    let fov = 60_f32.to_radians();
    let player_x = 3.0;
    let player_y = 3.0;

    // Create window & run event loop
    let window = Window::new_centered("Title", (640, 480)).unwrap();
    window.run_loop(MyWindowHandler {
        world,
        pov,
        fov,
        player_x,
        player_y,
        rect_height_top: 240.0 - 240.0 / len,
        rect_height_bot: 240.0 + 240.0 / len,
    });
}