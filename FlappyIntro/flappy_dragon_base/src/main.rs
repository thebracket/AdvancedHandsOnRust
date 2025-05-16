use bevy::{app::AppExit, prelude::*};
use my_library_docs::*;

//START: components
#[derive(Component)]
struct Flappy {//<callout id="flappy.basics.flappy" />
  gravity: f32,//<callout id="flappy.basics.gravity" />
}

#[derive(Component)]
struct Obstacle;//<callout id="flappy.basics.obstacle" />

#[derive(Resource)]
struct Assets {//<callout id="flappy.basics.assets" />
  dragon: Handle<Image>,
  wall: Handle<Image>,
}
//END: components

//START: main
fn main() {
  App::new()
    .add_plugins(DefaultPlugins.set(WindowPlugin {
      primary_window: Some(Window {//<callout id="flappy.basics.window_desc" />
      title: "Flappy Dragon - Bevy Edition".to_string(),
      resolution: bevy::window::WindowResolution::new(
        1024.0, 768.0
      ),
      ..default()
    }),
      ..default()
    }))
    .add_plugins(RandomPlugin)//<callout id="flappy.basics.random" />
    .add_systems(Startup, setup)
    .add_systems(Update, gravity)
    .add_systems(Update, flap)
    .add_systems(Update, clamp)
    .add_systems(Update, move_walls)
    .add_systems(Update, hit_wall)
    .run();
}
//END: main

//START: setup
fn setup(
  mut commands: Commands,
  asset_server: Res<AssetServer>,
  mut rng: ResMut<RandomNumberGenerator>,//<callout id="flappy.basics.setup_rng" />
) {
  let assets = Assets {//<callout id="flappy.basics.load_assets" />
    dragon: asset_server.load("flappy_dragon.png"),
    wall: asset_server.load("wall.png"),
  };

  commands.spawn(Camera2d::default());//<callout id="flappy.basics.2d_camera" />
  commands
    .spawn((
       Sprite::from_image(assets.dragon.clone()),//<callout id="flappy.basics.spawn_flappy" />
       Transform::from_xyz(-490.0, 0.0, 1.0),//<callout id="flappy.basics.spawn_flappy_pos" />
       Flappy { gravity: 0.0 }
    ));

  build_wall(&mut commands, assets.wall.clone(), rng.range(-5..5));//<callout id="flappy.basics.build_wall" />
  commands.insert_resource(assets);//<callout id="flappy.basics.store_assets" />
}
//END: setup

// START: build_wall
fn build_wall(
  commands: &mut Commands,
  wall_sprite: Handle<Image>,
  gap_y: i32,
) {
  for y in -12..=12 {//<callout id="flappy.basics.y_range" />
    if y < gap_y - 4 || y > gap_y + 4 {//<callout id="flappy.basics.gap_y" />
      commands
        .spawn((
            Sprite::from_image(wall_sprite.clone()),
            Transform::from_xyz(512.0, y as f32 * 32.0, 1.0),
            Obstacle,
          ));  
    }
  }
}
//END: build_wall

//START: gravity
fn gravity(mut query: Query<(&mut Flappy, &mut Transform)>) {
  if let Ok((mut flappy, mut transform)) = query.single_mut() {//<callout id="flappy.basics.get_flappy" />
    flappy.gravity += 0.1;//<callout id="flappy.basics.inc_gravity" />
    transform.translation.y -= flappy.gravity;//<callout id="flappy.basics.dec_pos" />
  }
}
//END: gravity

//START: flap
fn flap(keyboard: Res<ButtonInput<KeyCode>>, mut query: Query<&mut Flappy>) {
  if keyboard.pressed(KeyCode::Space) {
    if let Ok(mut flappy) = query.single_mut() {
      flappy.gravity = -5.0;//<callout id="flappy.basics.flap" />
    }
  }
}
//END: flap

//START: clamp
fn clamp(
  mut query: Query<&mut Transform, With<Flappy>>,
  mut exit: EventWriter<AppExit>,//<callout id="flappy.basics.exit" />
) {
  if let Ok(mut transform) = query.single_mut() {
    if transform.translation.y > 384.0 {
      transform.translation.y = 384.0;//<callout id="flappy.basics.at_the_top" />
    } else if transform.translation.y < -384.0 {
      exit.write(AppExit::Success);//<callout id="flappy.basics.send_exit" />
    }
  }
}
//END: clamp

//START: move_wall1
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
      rebuild = true;//<callout id="flappy.basics.need_rebuild" />
    }
  }
//END: move_wall1
//START: move_wall2
  if rebuild {
    for entity in delete.iter() {
      commands.entity(entity).despawn();
    }
    build_wall(&mut commands, assets.wall.clone(), rng.range(-5..5));
  }
}
//END: move_wall2

//START: hit_wall
fn hit_wall(
  player: Query<&Transform, With<Flappy>>,//<callout id="flappy.basics.find_player" />
  walls: Query<&Transform, With<Obstacle>>,//<callout id="flappy.basics.find_walls" />
  mut exit: EventWriter<AppExit>,
) {
  if let Ok(player) = player.single() {//<callout id="flappy.basics.just_player" />
    for wall in walls.iter() {//<callout id="flappy.basics.all_walls" />
      let distance = player.translation.distance(wall.translation);//<callout id="flappy.basics.distance" />
      if distance < 32.0 {
        exit.write(AppExit::Success);//<callout id="flappy.basics.end_if_hit" />
      }
    }
  }
}
//END: hit_wall