use rand::prelude::*;
use rusty_engine::prelude::*;
use std::default::Default;

#[derive(Default)]
struct Enemy {
    label: String,
    position: Vec2,
    direction: f32,
    amplitude: f32,
}

#[derive(Default)]
struct GameState {
    health: f32,
    direction: f32,
    speed: f32,
    score: i32,
    spawn_timer: Timer,
    player_hit: bool,
    enemies: Vec<Enemy>,
}

fn main() {
    let mut game = Game::new();
    game.window_settings(WindowDescriptor {
        title: "Boring Game".into(),
        width: 1920.0,
        height: 1080.0,
        ..Default::default()
    });

    let track_inner_sprite = game.add_sprite("track_inner", "track/track01.png");
    track_inner_sprite.collision = true;
    track_inner_sprite.layer = 0.0;

    let track_outer_sprite = game.add_sprite("track_outer", "track/track01_outer.png");
    track_outer_sprite.collision = true;
    track_outer_sprite.layer = 0.0;

    let player_sprite = game.add_sprite("player", SpritePreset::RacingCarGreen);
    player_sprite.scale = 0.5;
    player_sprite.translation = Vec2::new(0.0, 300.0);
    player_sprite.collision = true;
    player_sprite.layer = 100.0;

    let _ = game.add_text("speed", "");
    let _ = game.add_text("score", "");
    let _ = game.add_text("health", "");

    game.audio_manager
        .play_music(MusicPreset::WhimsicalPopsicle, 0.1);

    game.add_logic(player_movement_logic);
    game.add_logic(enemy_movement_logic);
    game.add_logic(collision_logic);
    game.add_logic(scoring_logic);
    game.add_logic(enemy_spawn_logic);
    game.add_logic(hud_logic);

    let initial_game_state = GameState {
        health: 100.0,
        spawn_timer: Timer::from_seconds(0.0, false),
        player_hit: false,
        enemies: vec![
            Enemy {
                label: "enemy_1".to_string(),
                position: Vec2::new(-150.0, 300.0),
                direction: UP,
                amplitude: 20.0,
            },
            Enemy {
                label: "enemy_2".to_string(),
                position: Vec2::new(0.0, -300.0),
                direction: LEFT,
                amplitude: 50.0,
            },
        ],
        ..Default::default()
    };

    game.run(initial_game_state);
}

const ACCELERATION: f32 = 10.0;
const ROTATION_SPEED: f32 = 5.0;

fn player_movement_logic(engine: &mut Engine, game_state: &mut GameState) {
    let player = engine.sprites.get_mut("player").unwrap();

    if engine.keyboard_state.pressed(KeyCode::Up) {
        game_state.speed += ACCELERATION;
    }
    if engine.keyboard_state.pressed(KeyCode::Down) {
        game_state.speed -= ACCELERATION;
    }
    if engine.keyboard_state.pressed(KeyCode::Left) {
        game_state.direction += ROTATION_SPEED * engine.delta_f32;
    }
    if engine.keyboard_state.pressed(KeyCode::Right) {
        game_state.direction -= ROTATION_SPEED * engine.delta_f32;
    }

    player.rotation = game_state.direction;

    player.translation.x += game_state.speed * engine.delta_f32 * game_state.direction.cos();
    player.translation.y += game_state.speed * engine.delta_f32 * game_state.direction.sin();
}

fn enemy_movement_logic(engine: &mut Engine, game_state: &mut GameState) {
    let time = engine.time_since_startup_f64;

    for enemy in &mut game_state.enemies {
        let sprite = match engine.sprites.get_mut(enemy.label.as_str()) {
            Some(s) => s,
            _ => {
                let new_sprite =
                    engine.add_sprite(enemy.label.clone(), SpritePreset::RacingBarrelRed);
                new_sprite.layer = 1.0;
                new_sprite.collision = true;
                new_sprite
            }
        };

        sprite.translation.x =
            enemy.position.x + enemy.direction.cos() * enemy.amplitude * time.sin() as f32;
        sprite.translation.y =
            enemy.position.y + enemy.direction.sin() * enemy.amplitude * time.sin() as f32;
    }
}

fn collision_logic(engine: &mut Engine, game_state: &mut GameState) {
    for collision_event in &engine.collision_events {
        println!(
            "Collision between: {} and {}, {}",
            collision_event.pair.0,
            collision_event.pair.1,
            match collision_event.state {
                CollisionState::Begin => "Begin",
                _ => "End",
            }
        );

        if collision_event.pair.one_starts_with("track_inner")
            && collision_event.pair.one_starts_with("player")
        {
            match collision_event.state {
                CollisionState::Begin => {
                    game_state.player_hit = true;
                    engine.audio_manager.play_sfx(SfxPreset::Impact1, 0.4);
                }
                _ => game_state.player_hit = false,
            }
        }

        if collision_event.pair.one_starts_with("track_outer")
            && collision_event.pair.one_starts_with("player")
        {
            match collision_event.state {
                CollisionState::End => {
                    game_state.player_hit = true;
                    engine.audio_manager.play_sfx(SfxPreset::Impact1, 0.4);
                }
                _ => game_state.player_hit = false,
            }
        }
    }
}

const HIT_RATE: f32 = 10.0;

fn scoring_logic(engine: &mut Engine, game_state: &mut GameState) {
    if game_state.player_hit {
        game_state.health -= HIT_RATE * engine.delta_f32;
    }

    for collision_event in engine.collision_events.drain(..) {
        if collision_event.pair.one_starts_with("enemy")
            && collision_event.pair.one_starts_with("player")
        {
            match collision_event.state {
                CollisionState::Begin => {
                    game_state.score += 10;
                    engine.audio_manager.play_sfx(SfxPreset::Confirmation1, 0.4);
                }
                _ => {}
            }
        }
    }
}

fn enemy_spawn_logic(engine: &mut Engine, game_state: &mut GameState) {
    if game_state.spawn_timer.tick(engine.delta).just_finished() {
        game_state.spawn_timer = Timer::from_seconds(thread_rng().gen_range(1.5..3.5), false);
        println!("Would spawn enemy");
    }
}

fn hud_logic(engine: &mut Engine, game_state: &mut GameState) {
    let speed_text = engine.texts.get_mut("speed").unwrap();
    speed_text.translation = Vec2::new(
        engine.window_dimensions.x / 2.0 - 200.0,
        engine.window_dimensions.y / 2.0 - speed_text.font_size - 5.0,
    );
    speed_text.value = format!("Speed {}", game_state.speed);

    let score_text = engine.texts.get_mut("score").unwrap();
    score_text.translation = Vec2::new(
        0.0,
        engine.window_dimensions.y / 2.0 - score_text.font_size - 5.0,
    );
    score_text.value = format!("Score {}", game_state.score);

    let health_text = engine.texts.get_mut("health").unwrap();
    health_text.translation = Vec2::new(
        -1.0 * engine.window_dimensions.x / 2.0 + 60.0,
        engine.window_dimensions.y / 2.0 - health_text.font_size - 5.0,
    );
    health_text.value = format!("Health {}", game_state.health as i32);
}
