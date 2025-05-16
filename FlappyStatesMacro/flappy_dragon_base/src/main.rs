use bevy::prelude::*;
use my_library_flappy_state_macro::*;

#[derive(Component)]
struct Flappy {
  gravity: f32,
}

#[derive(Component)]
struct Obstacle;

#[derive(Component)]
struct FlappyElement;

#[derive(Resource)]
struct Assets {
  dragon: Handle<Image>,
  wall: Handle<Image>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default, States)]
enum GamePhase {
  #[default] MainMenu,
  Flapping,
  GameOver
}

//START: main
fn main() {
  let mut app = App::new();

  add_phase!(app, GamePhase, GamePhase::Flapping,
    start => [ setup ],
    run => [ gravity, flap, clamp, move_walls, hit_wall ],
    exit => [ cleanup::<FlappyElement> ]
  );

  app.add_plugins(DefaultPlugins.set(WindowPlugin {
    primary_window: Some(Window {
    title: "Flappy Dragon - Bevy Edition".to_string(),
    resolution: bevy::window::WindowResolution::new(
      1024.0, 768.0
    ),
    ..default()
  }),
    ..default()
  }))
    .add_plugins(RandomPlugin)
    .add_plugins(GameStatePlugin::new(
      GamePhase::MainMenu, 
      GamePhase::Flapping, 
      GamePhase::GameOver)
    )
    .run();
}
//END: main

fn setup(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  mut rng: ResMut<RandomNumberGenerator>,
) {
  let assets = Assets {
    dragon: asset_server.load("flappy_dragon.png"),
    wall: asset_server.load("wall.png"),
  };

  commands.spawn(Camera2d::default()).insert(FlappyElement);
  commands
   .spawn((
     Sprite::from_image(assets.dragon.clone()),
       Transform::from_xyz(-490.0, 0.0, 1.0),
       FlappyElement,
       Flappy { gravity: 0.0 },
   ));

  build_wall(&mut commands, assets.wall.clone(), rng.range(-5..5));
  commands.insert_resource(assets);
}

fn build_wall(
  commands: &mut Commands,
  wall_sprite: Handle<Image>,
  gap_y: i32,
) {
  for y in -12..=12 {
    if y < gap_y - 4 || y > gap_y + 4 {
      commands
        .spawn((
          Sprite::from_image(wall_sprite.clone()),
            Transform::from_xyz(512.0, y as f32 * 32.0, 1.0),
            Obstacle,
            FlappyElement
        ));
    }
  }
}

fn gravity(mut query: Query<(&mut Flappy, &mut Transform)>) {
  if let Ok((mut flappy, mut transform)) = query.single_mut() {
    flappy.gravity += 0.1;
    transform.translation.y -= flappy.gravity;
  }
}

fn flap(keyboard: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Flappy>) {
  if keyboard.pressed(KeyCode::Space) {
    if let Ok(mut flappy) = query.single_mut() {
      flappy.gravity = -5.0;
    }
  }
}

fn clamp(
  mut query: Query<&mut Transform, With<Flappy>>,
  mut state: ResMut<NextState<GamePhase>>,
) {
  if let Ok(mut transform) = query.single_mut() {
    if transform.translation.y > 384.0 {
      transform.translation.y = 384.0;
    } else if transform.translation.y < -384.0 {
      state.set(GamePhase::GameOver);
    }
  }
}

fn move_walls(
  mut commands: Commands,
  mut query: Query<&mut Transform, With<Obstacle>>,
  delete: Query<Entity, With<Obstacle>>,
  assets: Res<Assets>,
  mut rng: ResMut<RandomNumberGenerator>,
) {
  let mut rebuild = false;
  for mut transform in query.iter_mut() {
    transform.translation.x -= 4.0;
    if transform.translation.x < -530.0 {
      rebuild = true;
    }
  }

  if rebuild {
    for entity in delete.iter() {
      commands.entity(entity).despawn();
    }
    build_wall(&mut commands, assets.wall.clone(), rng.range(-5..5));
  }
}

fn hit_wall(
  player: Query<&Transform, With<Flappy>>,
  walls: Query<&Transform, With<Obstacle>>,
  mut state: ResMut<NextState<GamePhase>>,
) {
  if let Ok(player) = player.single() {
    for wall in walls.iter() {
      let distance = player.translation.distance(wall.translation);
      if distance < 32.0 {
        state.set(GamePhase::GameOver);
      }
    }
  }
}