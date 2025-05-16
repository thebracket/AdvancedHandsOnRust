use bevy::prelude::*;
use my_library_flappy_animation::*;

#[derive(Component)]
struct Flappy {
  gravity: f32,
}

#[derive(Component)]
struct Obstacle;

#[derive(Component)]
struct FlappyElement;

//START: GameStatesLoading
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default, States)]
enum GamePhase {
  //START_HIGHLIGHT
  #[default] Loading,
  //END_HIGHLIGHT
  MainMenu,
  Flapping,
  GameOver
}
//END: GameStatesLoading

//START: MainLoad
fn main() -> anyhow::Result<()> {
  let mut app = App::new();

  add_phase!(app, GamePhase, GamePhase::Flapping,
    start => [ setup ],
    run => [ gravity, flap, clamp, move_walls, hit_wall, cycle_animations,
      continual_parallax ],
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
    //START: AssetMan
    .add_plugins(AssetManager::new()
      .add_image("dragon", "flappy_dragon.png")?
      .add_image("wall", "wall.png")?
      .add_sound("flap", "dragonflap.ogg")?
      .add_sound("crash", "crash.ogg")?
      //START_HIGHLIGHT
      .add_sprite_sheet("flappy", "flappy_sprite_sheet.png", 
        62.0, 65.0, 4, 1)?
      //END_HIGHLIGHT
    //END: AssetMan
    .add_image("bg_static", "rocky-far-mountains.png")?
      .add_image("bg_far", "rocky-nowater-far.png")?
      .add_image("bg_mid", "rocky-nowater-mid.png")?
      .add_image("bg_close", "rocky-nowater-close.png")?
    )
    //START: Animation1
    .insert_resource(
      Animations::new()
        .with_animation("Straight and Level", PerFrameAnimation::new(
          vec![
            AnimationFrame::new(2, 500, vec![AnimationOption::NextFrame]),
            AnimationFrame::new(3, 500, vec![AnimationOption::GoToFrame(0)]),
          ]
        ))
        //END: Animation1
        //START: Animation2
        .with_animation("Flapping", PerFrameAnimation::new(
          vec![
            AnimationFrame::new(0, 66, vec![AnimationOption::NextFrame, 
              AnimationOption::PlaySound("flap".to_string())]),
            AnimationFrame::new(1, 66, vec![AnimationOption::NextFrame]),
            AnimationFrame::new(2, 66, vec![AnimationOption::NextFrame]),
            AnimationFrame::new(3, 66, vec![AnimationOption::NextFrame]),
            AnimationFrame::new(2, 66, vec![AnimationOption::NextFrame]),
            AnimationFrame::new(1, 66, vec![
              AnimationOption::SwitchToAnimation(
                "Straight and Level".to_string())
              ]),
          ]
        ))//END: Animation2
    )
    .run();

  Ok(())
}
//END: MainLoad

//START: Setup
fn setup(
  mut commands: Commands,
  mut rng: ResMut<RandomNumberGenerator>,
  assets: Res<AssetStore>,
  loaded_assets: Res<LoadedAssets>,
) {
  commands.spawn(Camera2d::default()).insert(FlappyElement);
  //START_HIGHLIGHT
  spawn_animated_sprite!(assets, commands, "flappy", -490.0, 0.0, 10.0,
    "Straight and Level", Flappy { gravity: 0.0 }, FlappyElement);
  //END_HIGHLIGHT
  build_wall(&mut commands, &assets, rng.range(-5..5), &loaded_assets);
  //END: Setup

  //START: Parallax
  spawn_image!(assets, commands, "bg_static", 0.0, 0.0, 1.0, 
    &loaded_assets, FlappyElement);
  spawn_image!(assets, commands, "bg_far", 0.0, 0.0, 2.0, 
    &loaded_assets, FlappyElement, 
  ContinualParallax::new(1280.0, 66, Vec2::new(1.0, 0.0)));
  spawn_image!(assets, commands, "bg_far", 1280.0, 0.0, 2.0, 
    &loaded_assets, FlappyElement, 
  ContinualParallax::new(1280.0, 66, Vec2::new(1.0, 0.0)));
  spawn_image!(assets, commands, "bg_mid", 0.0, 0.0, 3.0, 
    &loaded_assets, FlappyElement, 
  ContinualParallax::new(1280.0, 33, Vec2::new(1.0, 0.0)));
  spawn_image!(assets, commands, "bg_mid", 1280.0, 0.0, 3.0, 
    &loaded_assets, FlappyElement, 
  ContinualParallax::new(1280.0, 33, Vec2::new(1.0, 0.0)));
  spawn_image!(assets, commands, "bg_close", 0.0, 0.0, 4.0, 
    &loaded_assets, FlappyElement, 
  ContinualParallax::new(1280.0, 16, Vec2::new(2.0, 0.0)));
  spawn_image!(assets, commands, "bg_close", 1280.0, 0.0, 4.0, 
    &loaded_assets, FlappyElement, 
  ContinualParallax::new(1280.0, 16, Vec2::new(2.0, 0.0)));
  //END: Parallax
}

fn build_wall(
  commands: &mut Commands,
  assets: &AssetStore,
  gap_y: i32,
  loaded_assets: &LoadedAssets,
) {
  for y in -12..=12 {
    if y < gap_y - 4 || y > gap_y + 4 {
      spawn_image!(assets, commands, "wall", 512.0, y as f32 * 32.0, 10.0, 
      loaded_assets, Obstacle, FlappyElement);
    }
  }
}

fn gravity(mut query: Query<(&mut Flappy, &mut Transform)>) {
  if let Ok((mut flappy, mut transform)) = query.single_mut() {
    flappy.gravity += 0.1;
    transform.translation.y -= flappy.gravity;
  }
}

//START: Flap
fn flap(
  keyboard: Res<ButtonInput<KeyCode>>, 
  mut query: Query<(&mut Flappy, &mut AnimationCycle)>,
    //START_HIGHLIGHT
  // Remove the audio reference
  //END_HIGHLIGHT

) 
{
  if keyboard.pressed(KeyCode::Space) {
    if let Ok((mut flappy, mut animation)) = query.single_mut() {
      flappy.gravity = -5.0;
      //START_HIGHLIGHT
      animation.switch("Flapping");
      //END_HIGHLIGHT
    }
  }
}
//END: Flap

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
  assets: Res<AssetStore>,
  mut rng: ResMut<RandomNumberGenerator>,
  loaded_assets: Res<LoadedAssets>,
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
    build_wall(&mut commands, &assets, rng.range(-5..5), &loaded_assets);
  }
}

fn hit_wall(
  player: Query<&Transform, With<Flappy>>,
  walls: Query<&Transform, With<Obstacle>>,
  mut state: ResMut<NextState<GamePhase>>,
  assets: Res<AssetStore>,
  loaded_assets: Res<LoadedAssets>,
  mut commands: Commands,
) {
  if let Ok(player) = player.single() {
    for wall in walls.iter() {
      let distance = player.translation.distance(wall.translation);
      if distance < 32.0 {
        assets.play("crash", &mut commands, &loaded_assets);
        state.set(GamePhase::GameOver);
      }
    }
  }
}