use bevy::prelude::*;
use my_library_flappy_state_generic::*;

#[derive(Component)]
struct Flappy {
  gravity: f32,
}

#[derive(Component)]
struct Obstacle;

#[derive(Resource)]
struct Assets {
  dragon: Handle<Image>,
  wall: Handle<Image>,
}

// START: GamePhase
#[derive(Clone, PartialEq, Eq, Debug, Hash, Default, States)]
enum GamePhase {
  MainMenu,
  #[default] Flapping,
  GameOver
}
//END: GamePhase

//START: FlappyElement
#[derive(Component)]
struct FlappyElement;
//END: FlappyElement

fn main() {
  //START: plugins
  App::new()
  .add_plugins(DefaultPlugins.set(WindowPlugin {
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
    //START_HIGHLIGHT
    .add_plugins(GameStatePlugin::<GamePhase>::new(
        GamePhase::MainMenu, 
        GamePhase::Flapping, 
        GamePhase::GameOver)
      )
    //END_HIGHLIGHT
    //END: plugins
    //START: system_set
    .add_systems(OnEnter(GamePhase::Flapping), setup)
    .add_systems(Update, (
      gravity, flap, clamp, move_walls, hit_wall
    ).run_if(in_state(GamePhase::Flapping)))
    .add_systems(OnExit(GamePhase::Flapping), cleanup::<FlappyElement>)
    .run();
    //END: system_set
}

fn setup(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  mut rng: ResMut<RandomNumberGenerator>,
) {
  println!("Setup");
  let assets = Assets {
    dragon: asset_server.load("flappy_dragon.png"),
    wall: asset_server.load("wall.png"),
  };

  //START: setup
  commands.spawn(Camera2d::default())
  //START_HIGHLIGHT
    .insert(FlappyElement);
  //END_HIGHLIGHT
  commands
    .spawn((
       Sprite::from_image(assets.dragon.clone()),
       Transform::from_xyz(-490.0, 0.0, 1.0),
       Flappy { gravity: 0.0 }
    ))
    //START_HIGHLIGHT
    .insert(FlappyElement);
    //END_HIGHLIGHT
  //END: setup

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
      //START: build_wall
      commands
        .spawn((
          Sprite::from_image(wall_sprite.clone()),
          Transform::from_xyz(512.0, y as f32 * 32.0, 1.0),
          Obstacle,
          //START_HIGHLIGHT
          FlappyElement
          //END_HIGHLIGHT
        ));
      //END: build_wall
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

//START: clamp
fn clamp(
  mut query: Query<&mut Transform, With<Flappy>>,
  //START_HIGHLIGHT
  mut state: ResMut<NextState<GamePhase>>,
  //END_HIGHLIGHT
) {
  if let Ok(mut transform) = query.single_mut() {
    if transform.translation.y > 384.0 {
      transform.translation.y = 384.0;
    } else if transform.translation.y < -384.0 {
      //START_HIGHLIGHT
      state.set(GamePhase::GameOver);
      //END_HIGHLIGHT
    }
  }
}
//END: clamp

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

//START: hit_wall
fn hit_wall(
  player: Query<&Transform, With<Flappy>>,
  walls: Query<&Transform, With<Obstacle>>,
  //START_HIGHLIGHT
  mut state: ResMut<NextState<GamePhase>>,
  //END_HIGHLIGHT
) {
  if let Ok(player) = player.single() {
    for wall in walls.iter() {
      let distance = player.translation.distance(wall.translation);
      if distance < 32.0 {
        //START_HIGHLIGHT
        state.set(GamePhase::GameOver);
        //END_HIGHLIGHT
      }
    }
  }
}
//END: hit_wall