use bevy::prelude::*;
use my_library::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default, States)]
enum GamePhase {
  #[default]
  Loading,
  MainMenu,
  Playing,
  GameOver,
}

#[derive(Component)]
struct GameElement;

#[derive(Component)]
struct Player;

fn main() -> anyhow::Result<()> {
  let mut app = App::new();
  //START: systems
  add_phase!(app, GamePhase, GamePhase::Playing,
    start => [ setup ],
    run => [ movement, end_game, physics_clock, sum_impulses, apply_gravity, apply_velocity,
      terminal_velocity.after(apply_velocity) ],
    exit => [ cleanup::<GameElement> ]
  );
  //END: systems

  app.add_event::<Impulse>();
  app.add_event::<PhysicsTick>();
  app
    .add_plugins(DefaultPlugins.set(WindowPlugin {
      primary_window: Some(Window {
        title: "Mars Base One".to_string(),
        resolution: bevy::window::WindowResolution::new(1024.0, 768.0),
        ..default()
      }),
      ..default()
    }))
    .add_plugins(RandomPlugin)
    //START: add_asset
    .add_plugins(GameStatePlugin::new(
      GamePhase::MainMenu,
      GamePhase::Playing,
      GamePhase::GameOver,
    ))
    //START_HIGHLIGHT
    .add_plugins(AssetManager::new().add_image("ship", "ship.png")?)
    //END_HIGHLIGHT
    .insert_resource(Animations::new())
    //END: add_asset
    .run();

  Ok(())
}

fn setup(
  mut commands: Commands,
  assets: Res<AssetStore>,
  loaded_assets: Res<LoadedAssets>,
) {
  //START: spawn_ship
  commands
    .spawn(Camera2d::default())
    .insert(GameElement);

    //START_HIGHLIGHT
  spawn_image!(
    assets,
    commands,
    "ship",
    0.0,
    0.0,
    1.0,
    &loaded_assets,
    GameElement,
    Player,
    Velocity::default(),
    PhysicsPosition::new(Vec2::new(0.0, 0.0)),
    ApplyGravity
);
  //END_HIGHLIGHT
  //END: spawn_ship
}

//START: movement
fn movement(
  keyboard: Res<ButtonInput<KeyCode>>,
  mut player_query: Query<(Entity, &mut Transform), With<Player>>,
  mut impulses: EventWriter<Impulse>,
) {
  let Ok((entity, mut transform)) = player_query.single_mut() else {
    return;
  };
  if keyboard.pressed(KeyCode::ArrowLeft) {
    transform.rotate(Quat::from_rotation_z(f32::to_radians(2.0)));// <callout id="mb1.rotate_left" />
  }
  if keyboard.pressed(KeyCode::ArrowRight) {
    transform.rotate(Quat::from_rotation_z(f32::to_radians(-2.0)));// <callout id="mb1.rotate_right" />
  }
  if keyboard.pressed(KeyCode::ArrowUp) {
    impulses.write(Impulse {// <callout id="mb1.impulse" />
      target: entity,
      amount: transform.local_y().as_vec3(),// <callout id="mb1.transform" />
      absolute: false,
      source: 1,
    });
  }
}
//END: movement

//START: terminal
fn terminal_velocity(mut player_query: Query<&mut Velocity, With<Player>>) {
  let Ok(mut velocity) = player_query.single_mut() else {
    return;
  };
  let v2 = velocity.0.truncate();
  if v2.length() > 5.0 {
    let v2 = v2.normalize() * 5.0;
    velocity.0.x = v2.x;
    velocity.0.y = v2.y;
  }
}
//END: terminal

//START: end_game
fn end_game(
  mut state: ResMut<NextState<GamePhase>>,
  player_query: Query<&Transform, With<Player>>,
) {
  let Ok(transform) = player_query.single() else {
    return;
  };
  if transform.translation.y < -384.0 || transform.translation.y > 384.0 || 
    transform.translation.x < -512.0 || transform.translation.x > 512.0
  {
    state.set(GamePhase::GameOver);
  }
}
//END: end_game