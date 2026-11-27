use crate::framebuffer::{Framebuffer, rgba_to_u32};
use crate::game::Maze;
use crate::player::Player;
use raylib::prelude::*;

pub struct Intersect {
    pub distance: f32,
    pub impact: char,
}

pub fn cast_ray(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    a: f32,
    init_pos: Vector2,
    draw_line: bool,
) -> Intersect {
    let mut d = 0.0;
    let max_distance = 1000.0;
    let step_size = 2.0; // Incrementado para mejor rendimiento

    // Pre-calcular color para el rayo
    let ray_color = rgba_to_u32(212, 255, 247, 100);

    // Pre-calcular cos y sin para evitar recálculos
    let cos_a = a.cos();
    let sin_a = a.sin();
    let init_x = player.pos.x + init_pos.x;
    let init_y = player.pos.y + init_pos.y;

    while d < max_distance {
        let x = init_x + d * cos_a;
        let y = init_y + d * sin_a;

        let maze_x = x as usize / block_size;
        let maze_y = y as usize / block_size;

        if maze_y >= maze.len() || maze_x >= maze[0].len() {
            break;
        }

        if maze[maze_y][maze_x] != ' ' {
            return Intersect {
                distance: d,
                impact: maze[maze_y][maze_x],
            };
        }

        if draw_line {
            framebuffer.set_pixel_fast(x as u32, y as u32, ray_color);
        }

        d += step_size;
    }

    Intersect {
        distance: max_distance,
        impact: ' ',
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    fn maze() -> Maze {
        ["+--+--+", "|     |", "|  +  |", "+--+--+"]
            .iter()
            .map(|row| row.chars().collect())
            .collect()
    }

    fn cast(x: f32, y: f32, a: f32) -> Intersect {
        let mut fb = Framebuffer::new(10, 10, Color::BLACK);
        let player = Player::new(Vector2::new(x, y));
        cast_ray(&mut fb, &maze(), &player, 30, a, Vector2::zero(), false)
    }

    #[test]
    fn a_ray_stops_at_the_first_wall() {
        // Desde el centro de la celda (1, 1) hacia la derecha: la pared '|' empieza en x = 180
        let hit = cast(45.0, 45.0, 0.0);
        assert_eq!(hit.impact, '|');
        assert!((hit.distance - 135.0).abs() <= 2.0, "{}", hit.distance);
    }

    #[test]
    fn a_ray_reports_which_wall_it_hit() {
        // Hacia abajo desde (1, 3) choca con el '+' del centro
        let hit = cast(3.5 * 30.0, 45.0, FRAC_PI_2);
        assert_eq!(hit.impact, '+');
        assert!((hit.distance - 15.0).abs() <= 2.0, "{}", hit.distance);
    }
}
