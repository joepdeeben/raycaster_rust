use speedy2d::color::Color;
use speedy2d::shape::Rectangle;
use speedy2d::window::{VirtualKeyCode, WindowHandler, WindowHelper};
use speedy2d::{Graphics2D, Window};
use std::collections::HashSet;

/// Checks if the given (x, y) coordinates collide with a wall in the world
fn check_collision(world: &[[i32; 12]; 11], x: i32, y: i32) -> bool {
    x >= 0 && x < world[0].len() as i32 && y >= 0 && y < world.len() as i32 && world[y as usize][x as usize] != 0
}

/// Casts a ray and returns the distance and collision point
fn cast_ray(world: &[[i32; 12]; 11], px: f32, py: f32, angle: f32) -> (f32, f32, f32) {
    let sin_angle = angle.sin();
    let cos_angle = angle.cos();
    let mut distance = 0.0;

    loop {
        distance += 0.01; // Smaller step for higher precision
        let x = px + distance * cos_angle;
        let y = py + distance * sin_angle;

        if x < 0.0 || x >= world[0].len() as f32 || y < 0.0 || y >= world.len() as f32 || check_collision(world, x.floor() as i32, y.floor() as i32) {
            return (x, y, distance);
        }
    }
}

/// Represents the window and game state
struct MyWindowHandler {
    world: [[i32; 12]; 11],
    pov: f32,
    fov: f32,
    player_x: f32,
    player_y: f32,
    keys_pressed: HashSet<VirtualKeyCode>, // Track pressed keys
}

impl WindowHandler for MyWindowHandler {
    fn on_key_down(
        &mut self,
        _helper: &mut WindowHelper<()>,
        key_code: Option<VirtualKeyCode>,
        _scancode: speedy2d::window::KeyScancode,
    ) {
        if let Some(key) = key_code {
            self.keys_pressed.insert(key);
        }
    }

    fn on_key_up(
        &mut self,
        _helper: &mut WindowHelper<()>,
        key_code: Option<VirtualKeyCode>,
        _scancode: speedy2d::window::KeyScancode,
    ) {
        if let Some(key) = key_code {
            self.keys_pressed.remove(&key);
        }
    }

    fn on_draw(&mut self, helper: &mut WindowHelper<()>, graphics: &mut Graphics2D) {
        // Handle movement based on pressed keys
        if self.keys_pressed.contains(&VirtualKeyCode::Left) {
            self.pov -= 5.0_f32.to_radians();
        }
        if self.keys_pressed.contains(&VirtualKeyCode::Right) {
            self.pov += 5.0_f32.to_radians();
        }
        if self.keys_pressed.contains(&VirtualKeyCode::Up) {
            self.player_x += self.pov.cos() * 0.1;
            self.player_y += self.pov.sin() * 0.1;
        }
        if self.keys_pressed.contains(&VirtualKeyCode::Down) {
            self.player_x -= self.pov.cos() * 0.1;
            self.player_y -= self.pov.sin() * 0.1;
        }

        graphics.clear_screen(Color::from_rgb(0.8, 0.9, 1.0));

        let tile_size = 64.0; // Each grid cell size
        let screen_width = 1280.0;
        let screen_height = 960.0;
        let num_rays = 320; // Higher resolution for walls
        let half_fov = self.fov / 2.0;

        // Render 3D walls
        for i in 0..num_rays {
            let ray_angle = self.pov - half_fov + (self.fov / num_rays as f32) * i as f32;
            let (_, _, distance) = cast_ray(&self.world, self.player_x, self.player_y, ray_angle);

            // Adjust the perceived distance to prevent fisheye distortion
            let corrected_distance = distance * (self.pov - ray_angle).cos();

            // Calculate wall height based on the distance
            let wall_height = (screen_height / corrected_distance) * 1.5; // Scale for better visuals
            let wall_top = (screen_height / 2.0) - (wall_height / 2.0);
            let wall_bottom = (screen_height / 2.0) + (wall_height / 2.0);

            // Wall slice width
            let slice_width = screen_width / num_rays as f32;

            // Draw the wall slice
            graphics.draw_rectangle(
                Rectangle::from_tuples(
                    (i as f32 * slice_width, wall_top),
                    ((i as f32 + 1.0) * slice_width, wall_bottom),
                ),
                Color::RED,
            );
        }

        // Draw the 2D mini-map
        let map_scale = 50.0; // Mini-map scaling factor
        let map_offset_x = 0.0;
        let map_offset_y = 0.0;

        for (row, row_data) in self.world.iter().enumerate() {
            for (col, &value) in row_data.iter().enumerate() {
                if value == 1 {
                    let x1 = col as f32 * map_scale + map_offset_x;
                    let y1 = row as f32 * map_scale + map_offset_y;
                    let x2 = x1 + map_scale;
                    let y2 = y1 + map_scale;
                    graphics.draw_rectangle(Rectangle::from_tuples((x1, y1), (x2, y2)), Color::GRAY);
                }
            }
        }

        // Draw player on the mini-map
        let player_x = self.player_x * map_scale + map_offset_x;
        let player_y = self.player_y * map_scale + map_offset_y;
        graphics.draw_rectangle(
            Rectangle::from_tuples((player_x - 2.0, player_y - 2.0), (player_x + 2.0, player_y + 2.0)),
            Color::BLUE,
        );

        // Draw rays on the mini-map
        for i in 0..num_rays {
            let ray_angle = self.pov - half_fov + (self.fov / num_rays as f32) * i as f32;
            let (ray_x, ray_y, _distance) = cast_ray(&self.world, self.player_x, self.player_y, ray_angle);

            graphics.draw_line(
                (player_x, player_y),
                (ray_x * map_scale + map_offset_x, ray_y * map_scale + map_offset_y),
                1.0,
                Color::GREEN,
            );
        }

        helper.request_redraw();
    }
}

fn main() {
    let world =
        [
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            [1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 0, 0, 1, 0, 1, 1, 1, 1, 0, 1],
            [1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 1],
            [1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 0, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1],
            [1, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 1],
            [1, 0, 1, 0, 1, 1, 1, 0, 1, 1, 0, 1],
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
            [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        ];



    let window = Window::new_centered("Raycasting Renderer", (1280, 960)).unwrap();

    window.run_loop(MyWindowHandler {
        world,
        pov: 90.0_f32.to_radians(),
        fov: 60.0_f32.to_radians(),
        player_x: 3.0,
        player_y: 3.0,
        keys_pressed: HashSet::new(),
    });
}
