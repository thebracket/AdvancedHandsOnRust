//START: setup
use bevy::prelude::*;
use my_library::*;
//END: setup

//START: phase_element
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
//END: phase_element

//START: main
fn main() -> anyhow::Result<()> {
  let mut app = App::new();
  add_phase!(app, GamePhase, GamePhase::Playing,
    start => [ setup ],
    run => [ end_game, physics_clock, sum_impulses, apply_gravity, 
      apply_velocity ],
    exit => [ cleanup::<GameElement> ]
  );

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
    .add_plugins(GameStatePlugin::new(
      GamePhase::MainMenu,
      GamePhase::Playing,
      GamePhase::GameOver,
    ))
    .add_plugins(AssetManager::new())
    .insert_resource(Animations::new())
    .run();

  Ok(())
}
//END: main

//START: setup2
fn setup(
  mut commands: Commands,
  assets: Res<AssetStore>,
  loaded_assets: Res<LoadedAssets>,
) {
  commands
    .spawn(Camera2d::default())
    .insert(GameElement);
}

fn end_game(
  mut state: ResMut<NextState<GamePhase>>,
  assets: Res<AssetStore>,
) {  
  //let _ = state.set(GamePhase::GameOver);// <callout id="mb0.end_game" />
}
//END: setup2