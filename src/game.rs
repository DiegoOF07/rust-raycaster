use raylib::prelude::*;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::cast::cast_ray;
use crate::framebuffer::{Framebuffer, rgba_to_u32};
use crate::gameplay::SpriteKind;
use crate::player::Player;
use crate::sprites::Sprite;

pub type Maze = Vec<Vec<char>>;

#[derive(PartialEq)]
pub enum GameState {
    Menu,
    Playing,
    Exiting,
}

pub fn load_maze_with_sprites(filename: &str, block_size: usize) -> (Maze, Vec<Sprite>) {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    let mut maze: Maze = Vec::new();
    let mut sprites: Vec<Sprite> = Vec::new();

    for (row_idx, line) in reader.lines().enumerate() {
        let mut row: Vec<char> = Vec::new();
        for (col_idx, ch) in line.unwrap().chars().enumerate() {
            match ch {
                'O' => {
                    let world_x = col_idx as f32 + 0.5;
                    let world_y = row_idx as f32 + 0.5;
                    sprites.push(Sprite::new_static(
                        Vector2::new(world_x * block_size as f32, world_y * block_size as f32),
                        "key".to_string(),
                        8.0,
                        SpriteKind::Key,
                    ));
                    row.push(' ');
                }
                'A' => {
                    let world_x = col_idx as f32 + 0.5;
                    let world_y = row_idx as f32 + 0.5;
                    sprites.push(Sprite::new_static(
                        Vector2::new(world_x * block_size as f32, world_y * block_size as f32),
                        "spike".to_string(),
                        12.0,
                        SpriteKind::Spike,
                    ));
                    row.push(' ');
                }
                'F' => {
                    let world_x = col_idx as f32 + 0.5;
                    let world_y = row_idx as f32 + 0.5;
                    sprites.push(Sprite::new_animated(
                        Vector2::new(world_x * block_size as f32, world_y * block_size as f32),
                        vec![
                            "fire1".to_string(),
                            "fire2".to_string(),
                            "fire3".to_string(),
                        ],
                        0.2,
                        12.0,
                        SpriteKind::Fire,
                    ));
                    row.push(' ');
                }

                'H' => {
                    let world_x = col_idx as f32 + 0.5;
                    let world_y = row_idx as f32 + 0.5;
                    sprites.push(Sprite::new_animated(
                        Vector2::new(world_x * block_size as f32, world_y * block_size as f32),
                        vec![
                            "heal1".to_string(),
                            "heal2".to_string(),
                            "heal3".to_string(),
                            "heal4".to_string(),
                        ],
                        0.2,
                        12.0,
                        SpriteKind::Heal,
                    ));
                    row.push(' ');
                }
                _ => {
                    row.push(ch);
                }
            }
        }
        maze.push(row);
    }

    (maze, sprites)
}

pub fn render_maze(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    pos: Vector2,
    with_rays: bool,
) {
    // Pre-calcular colores como u32 para mejor rendimiento
    let black_color = rgba_to_u32(0, 0, 0, 255);
    let red_color = rgba_to_u32(255, 0, 0, 255);

    for (row_index, row) in maze.iter().enumerate() {
        for (col_index, &cell) in row.iter().enumerate() {
            let xo = (col_index * block_size) + pos.x as usize;
            let yo = (row_index * block_size) + pos.y as usize;
            draw_cell_optimized(
                framebuffer,
                xo,
                yo,
                block_size,
                cell,
                black_color,
                red_color,
            );
        }
    }

    if with_rays {
        let num_rays = 25;
        for i in 0..num_rays {
            let current_ray = i as f32 / num_rays as f32;
            let a = player.a - (player.fov / 2.0) + (player.fov * current_ray);
            cast_ray(framebuffer, maze, player, block_size, a, pos, true);
        }
    }

    let scale = block_size as f32 / 30.0;
    let scaled_pos = Vector2::new(player.pos.x * scale, player.pos.y * scale);
    let triangle_size = block_size as f32 * 0.5;

    let fake_player = Player {
        pos: scaled_pos,
        a: player.a,
        fov: player.fov,
    };

    fake_player.draw_player_triangle(framebuffer, pos, triangle_size, Color::WHITESMOKE);
}

// Función optimizada para dibujar celdas
fn draw_cell_optimized(
    framebuffer: &mut Framebuffer,
    xo: usize,
    yo: usize,
    block_size: usize,
    cell: char,
    black_color: u32,
    red_color: u32,
) {
    let color = match cell {
        ' ' => black_color,
        _ => red_color,
    };

    // Optimización: dibujar rectángulo sólido usando loops optimizados
    let end_x = xo + block_size;
    let end_y = yo + block_size;

    for y in yo..end_y {
        if y < framebuffer.height as usize {
            for x in xo..end_x {
                if x < framebuffer.width as usize {
                    framebuffer.set_pixel_fast(x as u32, y as u32, color);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gameplay::SpriteKind;
    use std::collections::VecDeque;

    const LEVELS: [&str; 3] = [
        "./levels/level1.txt",
        "./levels/level2.txt",
        "./levels/level3.txt",
    ];

    fn cell_of(pos: Vector2) -> (usize, usize) {
        ((pos.y / 30.0) as usize, (pos.x / 30.0) as usize)
    }

    #[test]
    fn every_level_loads_with_one_key_and_open_cells_under_its_sprites() {
        for level in LEVELS {
            let (maze, sprites) = load_maze_with_sprites(level, 30);
            let keys = sprites.iter().filter(|s| s.kind == SpriteKind::Key).count();
            assert_eq!(keys, 1, "{level}");
            assert!(
                maze.iter().all(|row| row.len() == maze[0].len()),
                "{level}: filas de distinto ancho"
            );
            for sprite in &sprites {
                let (r, c) = cell_of(sprite.pos);
                assert_eq!(
                    maze[r][c], ' ',
                    "{level}: sprite sobre una pared en ({r}, {c})"
                );
            }
        }
    }

    #[test]
    fn the_key_can_be_reached_from_the_start_in_every_level() {
        for level in LEVELS {
            let (maze, sprites) = load_maze_with_sprites(level, 30);
            let key = sprites
                .iter()
                .find(|s| s.kind == SpriteKind::Key)
                .expect("llave");
            let goal = cell_of(key.pos);
            // El jugador empieza en la celda (1, 1)
            let mut seen = vec![vec![false; maze[0].len()]; maze.len()];
            let mut queue = VecDeque::from([(1usize, 1usize)]);
            seen[1][1] = true;
            while let Some((r, c)) = queue.pop_front() {
                for (dr, dc) in [(0i32, 1i32), (1, 0), (0, -1), (-1, 0)] {
                    let (nr, nc) = (r as i32 + dr, c as i32 + dc);
                    if nr < 0 || nc < 0 || nr as usize >= maze.len() || nc as usize >= maze[0].len()
                    {
                        continue;
                    }
                    let (nr, nc) = (nr as usize, nc as usize);
                    if maze[nr][nc] == ' ' && !seen[nr][nc] {
                        seen[nr][nc] = true;
                        queue.push_back((nr, nc));
                    }
                }
            }
            assert!(
                seen[goal.0][goal.1],
                "{level}: la llave en {goal:?} no se alcanza desde el inicio"
            );
        }
    }
}
