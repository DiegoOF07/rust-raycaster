// gameplay.rs
// Vida, trampas, curaciones y llave: las reglas del juego, sin ventana ni audio

use crate::sprites::Sprite;
use raylib::prelude::Vector2;

pub const MAX_HEALTH: i32 = 100;
/// Vida que quita una trampa (pinchos o fuego)
pub const TRAP_DAMAGE: i32 = 20;
/// Vida que devuelve una curación
pub const HEAL_AMOUNT: i32 = 25;
/// Segundos sin recibir daño después de un golpe
pub const INVULNERABLE_SECS: f32 = 1.0;
/// Distancia, en unidades del mundo, a la que se toca un sprite: media celda (una celda mide 30)
pub const PICKUP_RADIUS: f32 = 15.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SpriteKind {
    Key,
    Spike,
    Fire,
    Heal,
}

/// Lo que pasó en un cuadro; main.rs lo convierte en sonidos y efectos
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Event {
    Hurt,
    Healed,
    Won,
    Died,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    Won,
    Lost,
}

pub struct Status {
    pub health: i32,
    pub outcome: Option<Outcome>,
    invulnerable: f32,
}

impl Status {
    pub fn new() -> Self {
        Status {
            health: MAX_HEALTH,
            outcome: None,
            invulnerable: 0.0,
        }
    }

    /// Fracción de vida restante, de 0 a 1
    pub fn health_fraction(&self) -> f32 {
        self.health.clamp(0, MAX_HEALTH) as f32 / MAX_HEALTH as f32
    }

    /// Avanza un cuadro: aplica lo que toca el jugador y quita del mapa lo que recoge
    pub fn update(
        &mut self,
        player_pos: Vector2,
        sprites: &mut Vec<Sprite>,
        dt: f32,
    ) -> Vec<Event> {
        let mut events = Vec::new();
        if self.outcome.is_some() {
            return events;
        }
        self.invulnerable = (self.invulnerable - dt).max(0.0);

        let mut picked = Vec::new();
        for (i, sprite) in sprites.iter().enumerate() {
            let d = sprite.pos - player_pos;
            if d.x * d.x + d.y * d.y > PICKUP_RADIUS * PICKUP_RADIUS {
                continue;
            }
            match sprite.kind {
                SpriteKind::Key => {
                    picked.push(i);
                    self.outcome = Some(Outcome::Won);
                    events.push(Event::Won);
                }
                SpriteKind::Heal if self.health < MAX_HEALTH => {
                    picked.push(i);
                    self.health = (self.health + HEAL_AMOUNT).min(MAX_HEALTH);
                    events.push(Event::Healed);
                }
                SpriteKind::Heal => {}
                SpriteKind::Spike | SpriteKind::Fire => {
                    if self.invulnerable == 0.0 {
                        self.health -= TRAP_DAMAGE;
                        self.invulnerable = INVULNERABLE_SECS;
                        events.push(Event::Hurt);
                        if self.health <= 0 {
                            self.outcome = Some(Outcome::Lost);
                            events.push(Event::Died);
                        }
                    }
                }
            }
            if self.outcome.is_some() {
                break;
            }
        }
        for i in picked.into_iter().rev() {
            sprites.remove(i);
        }

        events
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sprites::Sprite;
    use raylib::prelude::Vector2;

    const DT: f32 = 1.0 / 60.0;

    fn sprite(kind: SpriteKind, x: f32, y: f32) -> Sprite {
        Sprite::new_static(Vector2::new(x, y), "x".to_string(), 10.0, kind)
    }

    fn here() -> Vector2 {
        Vector2::new(45.0, 45.0)
    }

    #[test]
    fn a_trap_hurts_once_and_then_gives_a_moment_of_invulnerability() {
        let mut status = Status::new();
        let mut sprites = vec![sprite(SpriteKind::Spike, 45.0, 45.0)];
        assert_eq!(status.update(here(), &mut sprites, DT), vec![Event::Hurt]);
        assert_eq!(status.health, MAX_HEALTH - TRAP_DAMAGE);

        // Medio segundo encima de la trampa: no vuelve a doler
        for _ in 0..30 {
            assert!(status.update(here(), &mut sprites, DT).is_empty());
        }
        assert_eq!(status.health, MAX_HEALTH - TRAP_DAMAGE);

        // Pasado el segundo de invulnerabilidad, duele otra vez
        let events: Vec<Event> = (0..40)
            .flat_map(|_| status.update(here(), &mut sprites, DT))
            .collect();
        assert_eq!(events, vec![Event::Hurt]);
        assert_eq!(status.health, MAX_HEALTH - 2 * TRAP_DAMAGE);
        assert_eq!(sprites.len(), 1, "las trampas no desaparecen");
    }

    #[test]
    fn fire_hurts_like_a_spike() {
        let mut status = Status::new();
        let mut sprites = vec![sprite(SpriteKind::Fire, 45.0, 45.0)];
        assert_eq!(status.update(here(), &mut sprites, DT), vec![Event::Hurt]);
    }

    #[test]
    fn losing_all_health_ends_the_game() {
        let mut status = Status::new();
        status.health = TRAP_DAMAGE;
        let mut sprites = vec![sprite(SpriteKind::Spike, 45.0, 45.0)];
        assert_eq!(
            status.update(here(), &mut sprites, DT),
            vec![Event::Hurt, Event::Died]
        );
        assert_eq!(status.outcome, Some(Outcome::Lost));
        // Terminado el juego, ya no pasa nada
        assert!(status.update(here(), &mut sprites, 5.0).is_empty());
    }

    #[test]
    fn healing_restores_health_up_to_the_maximum_and_is_used_up() {
        let mut status = Status::new();
        status.health = MAX_HEALTH - 10;
        let mut sprites = vec![sprite(SpriteKind::Heal, 45.0, 45.0)];
        assert_eq!(status.update(here(), &mut sprites, DT), vec![Event::Healed]);
        assert_eq!(status.health, MAX_HEALTH);
        assert!(sprites.is_empty());
    }

    #[test]
    fn healing_is_left_on_the_floor_at_full_health() {
        let mut status = Status::new();
        let mut sprites = vec![sprite(SpriteKind::Heal, 45.0, 45.0)];
        assert!(status.update(here(), &mut sprites, DT).is_empty());
        assert_eq!(sprites.len(), 1);
    }

    #[test]
    fn picking_up_the_key_wins_the_level() {
        let mut status = Status::new();
        let mut sprites = vec![sprite(SpriteKind::Key, 45.0, 45.0)];
        assert_eq!(status.update(here(), &mut sprites, DT), vec![Event::Won]);
        assert_eq!(status.outcome, Some(Outcome::Won));
        assert!(sprites.is_empty());
    }

    #[test]
    fn sprites_out_of_reach_do_nothing() {
        let mut status = Status::new();
        let mut sprites = vec![
            sprite(SpriteKind::Spike, 45.0 + PICKUP_RADIUS + 1.0, 45.0),
            sprite(SpriteKind::Key, 45.0, 45.0 - PICKUP_RADIUS - 1.0),
        ];
        assert!(status.update(here(), &mut sprites, DT).is_empty());
        assert_eq!(status.health, MAX_HEALTH);
    }
}
