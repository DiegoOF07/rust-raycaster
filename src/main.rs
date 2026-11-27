mod audio_manager;
mod cast;
mod events;
mod framebuffer;
mod game;
mod gameplay;
mod player;
mod renderer;
mod sprites;
mod texture;

use audio_manager::AudioManager;
use events::process_events;
use framebuffer::Framebuffer;
use game::{GameState, load_maze_with_sprites, render_maze};
use gameplay::{Event, Outcome, Status};
use player::Player;
use raylib::prelude::*;
use renderer::{render_world, render_world_with_textures_downscale};
use sprites::render_sprites;
use texture::TextureManager;

fn main() {
    let window_width = 930;
    let window_height = 630;
    let block_size = 30_usize;
    let wall_textures = [
        "./textures/wall1.png",
        "./textures/wall2.png",
        "./textures/wall3.png",
    ];
    let floor_textures = [
        "./textures/floor1.png",
        "./textures/floor2.jpg",
        "./textures/floor3.png",
    ];
    let level_musics = [
        "./audio/music/song1.ogg",
        "./audio/music/song2.mp3",
        "./audio/music/song3.wav",
    ];

    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Raycaster Game")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    window.set_target_fps(60);
    window.hide_cursor();
    let mut texture_manager = TextureManager::new();

    texture_manager.generate_default_textures();
    texture_manager
        .load_sprite_texture(
            "key",
            "./textures/sprites/key.png",
            &mut window,
            &raylib_thread,
        )
        .ok();
    texture_manager
        .load_sprite_texture(
            "spike",
            "./textures/sprites/spike.png",
            &mut window,
            &raylib_thread,
        )
        .ok();
    texture_manager
        .load_sprite_texture(
            "fire1",
            "./textures/sprites/fire1.png",
            &mut window,
            &raylib_thread,
        )
        .ok();
    texture_manager
        .load_sprite_texture(
            "fire2",
            "./textures/sprites/fire2.png",
            &mut window,
            &raylib_thread,
        )
        .ok();
    texture_manager
        .load_sprite_texture(
            "fire3",
            "./textures/sprites/fire3.png",
            &mut window,
            &raylib_thread,
        )
        .ok();
    texture_manager
        .load_sprite_texture(
            "heal1",
            "./textures/sprites/heal1.png",
            &mut window,
            &raylib_thread,
        )
        .ok();
    texture_manager
        .load_sprite_texture(
            "heal2",
            "./textures/sprites/heal2.png",
            &mut window,
            &raylib_thread,
        )
        .ok();
    texture_manager
        .load_sprite_texture(
            "heal3",
            "./textures/sprites/heal3.png",
            &mut window,
            &raylib_thread,
        )
        .ok();
    texture_manager
        .load_sprite_texture(
            "heal4",
            "./textures/sprites/heal4.png",
            &mut window,
            &raylib_thread,
        )
        .ok();

    let mut audio_manager = AudioManager::new();

    // Cargar y mantener la textura de fondo del menú como Texture2D de Raylib
    let menu_bg_texture = match Image::load_image("./textures/menu_bg.jpg") {
        Ok(img) => Some(
            window
                .load_texture_from_image(&raylib_thread, &img)
                .unwrap(),
        ),
        Err(_) => {
            println!("No se pudo cargar la imagen de fondo del menú, usando color sólido");
            None
        }
    };

    let mut game_state = GameState::Menu;
    let level_files = [
        "./levels/level1.txt",
        "./levels/level2.txt",
        "./levels/level3.txt",
    ];
    let level_names = ["Nivel 1", "Nivel 2", "Nivel 3"];
    let mut selected_level = 0;

    audio_manager.play_music("./audio/music/menu.ogg");
    while !window.window_should_close() && game_state != GameState::Exiting {
        match game_state {
            GameState::Menu => {
                let mut d = window.begin_drawing(&raylib_thread);

                // Renderizar fondo
                if let Some(bg_texture) = &menu_bg_texture {
                    // Calcular escalado para que la imagen cubra toda la pantalla
                    let scale_x = window_width as f32 / bg_texture.width as f32;
                    let scale_y = window_height as f32 / bg_texture.height as f32;
                    let scale = scale_x.max(scale_y); // Usar el mayor para cubrir toda la pantalla

                    // Centrar la imagen
                    let scaled_width = bg_texture.width as f32 * scale;
                    let scaled_height = bg_texture.height as f32 * scale;
                    let offset_x = (window_width as f32 - scaled_width) / 2.0;
                    let offset_y = (window_height as f32 - scaled_height) / 2.0;

                    d.draw_texture_ex(
                        bg_texture,
                        Vector2::new(offset_x, offset_y),
                        0.0,
                        scale,
                        Color::WHITE,
                    );
                } else {
                    // Fondo alternativo si no se puede cargar la imagen
                    d.clear_background(Color::new(20, 20, 40, 255));
                }

                // Overlay semi-transparente para mejorar legibilidad
                d.draw_rectangle(0, 0, window_width, window_height, Color::new(0, 0, 0, 100));

                // Título con contorno para mejor visibilidad
                let title_text = ">> Horror Game <<";
                let title_x = window_width / 2 - 150;
                let title_y = 100;

                // Contorno del título
                for dx in -2..=2 {
                    for dy in -2..=2 {
                        if dx != 0 || dy != 0 {
                            d.draw_text(title_text, title_x + dx, title_y + dy, 30, Color::BLACK);
                        }
                    }
                }
                d.draw_text(title_text, title_x, title_y, 30, Color::RAYWHITE);

                // Instrucciones con contorno
                let instruction_text = "Selecciona un nivel:";
                let inst_x = window_width / 2 - 150;
                let inst_y = 180;

                // Contorno de instrucciones
                d.draw_text(instruction_text, inst_x + 1, inst_y + 1, 22, Color::BLACK);
                d.draw_text(instruction_text, inst_x, inst_y, 22, Color::LIGHTGRAY);

                // Dibujar opciones de nivel con selección y contorno
                for (i, name) in level_names.iter().enumerate() {
                    let color = if i == selected_level {
                        Color::YELLOW
                    } else {
                        Color::GRAY
                    };
                    let marker = if i == selected_level { ">" } else { " " };
                    let level_text = format!("{} {}", marker, name);
                    let level_x = window_width / 2 - 100;
                    let level_y = 220 + i as i32 * 40;

                    // Contorno para opciones de nivel
                    d.draw_text(&level_text, level_x + 1, level_y + 1, 28, Color::BLACK);
                    d.draw_text(&level_text, level_x, level_y, 28, color);
                }

                // Instrucciones de control con contorno
                let control_texts = [
                    "Usa W/S para cambiar nivel",
                    "ENTER para jugar | ESC para salir",
                ];

                for (i, text) in control_texts.iter().enumerate() {
                    let ctrl_x = window_width / 2 - 150;
                    let ctrl_y = 370 + i as i32 * 30;

                    d.draw_text(text, ctrl_x + 1, ctrl_y + 1, 18, Color::BLACK);
                    d.draw_text(text, ctrl_x, ctrl_y, 18, Color::DARKGRAY);
                }

                // Navegación con W/S
                if d.is_key_pressed(KeyboardKey::KEY_W) {
                    selected_level = selected_level.saturating_sub(1);
                }
                if d.is_key_pressed(KeyboardKey::KEY_S) && selected_level < level_files.len() - 1 {
                    selected_level += 1;
                }

                if d.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    game_state = GameState::Playing;
                } else if d.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
                    game_state = GameState::Exiting;
                }
            }

            GameState::Playing => {
                let framebuffer_width = 930;
                let framebuffer_height = 630;
                audio_manager.play_music(level_musics[selected_level]);
                texture_manager
                    .load_wall_texture(
                        '-',
                        wall_textures[selected_level],
                        &mut window,
                        &raylib_thread,
                    )
                    .ok();
                texture_manager
                    .load_wall_texture(
                        '|',
                        wall_textures[selected_level],
                        &mut window,
                        &raylib_thread,
                    )
                    .ok();
                texture_manager
                    .load_wall_texture(
                        '+',
                        wall_textures[selected_level],
                        &mut window,
                        &raylib_thread,
                    )
                    .ok();
                texture_manager
                    .load_floor_texture(floor_textures[selected_level], &mut window, &raylib_thread)
                    .ok();

                let (maze, mut sprites) =
                    load_maze_with_sprites(level_files[selected_level], block_size);
                let mut player = Player::new(Vector2::new(
                    1.5 * block_size as f32,
                    1.5 * block_size as f32,
                ));
                let mut framebuffer =
                    Framebuffer::new(framebuffer_width, framebuffer_height, Color::BLACK);

                let mut mode = "3D";
                let mut use_textures = true;
                let mut status = Status::new();
                // Intensidad del destello rojo al recibir daño, de 1 a 0
                let mut hurt_flash = 0.0_f32;

                window.set_mouse_position(Vector2::new(
                    window_width as f32 / 2.0,
                    window_height as f32 / 2.0,
                ));
                let mut last_mouse_x = window.get_mouse_x();

                while !window.window_should_close() {
                    let dt = window.get_frame_time();
                    let back_to_menu = window.is_key_pressed(KeyboardKey::KEY_P)
                        || (status.outcome.is_some()
                            && window.is_key_pressed(KeyboardKey::KEY_ENTER));
                    if window.is_key_pressed(KeyboardKey::KEY_M) {
                        mode = if mode == "2D" { "3D" } else { "2D" };
                    }
                    if window.is_key_pressed(KeyboardKey::KEY_T) {
                        use_textures = !use_textures;
                    }

                    framebuffer.clear();
                    if mode == "2D" {
                        render_maze(
                            &mut framebuffer,
                            &maze,
                            &player,
                            block_size,
                            Vector2::new(0.0, 0.0),
                            true,
                        );
                    } else {
                        if use_textures {
                            let depth_buffer = render_world_with_textures_downscale(
                                &mut framebuffer,
                                &maze,
                                &player,
                                block_size,
                                &texture_manager,
                            );
                            render_sprites(
                                &mut framebuffer,
                                &mut sprites,
                                &player,
                                &texture_manager,
                                &depth_buffer,
                            );
                        } else {
                            render_world(&mut framebuffer, &maze, &player, block_size);
                        }
                        render_maze(
                            &mut framebuffer,
                            &maze,
                            &player,
                            block_size - 20,
                            Vector2::new((window_width - 310) as f32, 0.0),
                            false,
                        );
                    }
                    // Al terminar el nivel el jugador queda quieto hasta volver al menú
                    if status.outcome.is_none() {
                        let before = player.pos;
                        process_events(
                            &mut window,
                            &mut player,
                            &maze,
                            &mut last_mouse_x,
                            block_size,
                        );
                        audio_manager.set_footsteps(player.pos != before);
                        for event in status.update(player.pos, &mut sprites, dt) {
                            match event {
                                Event::Hurt => {
                                    audio_manager.play_hurt();
                                    hurt_flash = 1.0;
                                }
                                Event::Won | Event::Died => audio_manager.set_footsteps(false),
                                Event::Healed => {}
                            }
                        }
                    }
                    hurt_flash = (hurt_flash - dt * 3.0).max(0.0);

                    framebuffer.swap_buffers(&mut window, &raylib_thread, |d| {
                        d.draw_text(&format!("FPS: {}", d.get_fps()), 10, 10, 20, Color::WHITE);
                        d.draw_text(&format!("Modo: {}", mode), 10, 35, 16, Color::WHITE);
                        d.draw_text(
                            &format!("Texturas: {}", if use_textures { "ON" } else { "OFF" }),
                            10,
                            55,
                            16,
                            Color::WHITE,
                        );
                        d.draw_text(
                            "M: Cambiar modo | T: Toggle texturas | P: Menú",
                            10,
                            75,
                            14,
                            Color::LIGHTGRAY,
                        );
                        draw_hud(d, &status, hurt_flash, window_width, window_height);
                    });

                    // Se sale después de dibujar: raylib actualiza el teclado al cerrar el cuadro,
                    // y si no el menú vería la misma tecla recién presionada
                    if back_to_menu {
                        audio_manager.set_footsteps(false);
                        audio_manager.play_music("./audio/music/menu.ogg");
                        window.show_cursor();
                        game_state = GameState::Menu;
                        break;
                    }
                }
            }

            GameState::Exiting => break,
        }
    }
}

/// Barra de vida, destello de daño y pantallas de victoria o derrota
fn draw_hud(d: &mut RaylibDrawHandle, status: &Status, hurt_flash: f32, width: i32, height: i32) {
    if hurt_flash > 0.0 {
        d.draw_rectangle(
            0,
            0,
            width,
            height,
            Color::new(200, 0, 0, (hurt_flash * 90.0) as u8),
        );
    }

    let (x, y, w, h) = (20, height - 40, 220, 18);
    d.draw_rectangle(x - 2, y - 2, w + 4, h + 4, Color::new(0, 0, 0, 180));
    d.draw_rectangle(
        x,
        y,
        (w as f32 * status.health_fraction()) as i32,
        h,
        Color::new(190, 30, 30, 255),
    );
    d.draw_text(
        &format!("Vida {}", status.health.max(0)),
        x + 6,
        y + 2,
        14,
        Color::WHITE,
    );

    if let Some(outcome) = status.outcome {
        let (title, color) = match outcome {
            Outcome::Won => ("¡Encontraste la llave!", Color::new(120, 220, 120, 255)),
            Outcome::Lost => ("Game over", Color::new(230, 60, 60, 255)),
        };
        d.draw_rectangle(0, 0, width, height, Color::new(0, 0, 0, 170));
        let title_x = (width - d.measure_text(title, 40)) / 2;
        d.draw_text(title, title_x, height / 2 - 40, 40, color);
        let hint = "ENTER para volver al menú";
        let hint_x = (width - d.measure_text(hint, 20)) / 2;
        d.draw_text(hint, hint_x, height / 2 + 15, 20, Color::LIGHTGRAY);
    }
}
