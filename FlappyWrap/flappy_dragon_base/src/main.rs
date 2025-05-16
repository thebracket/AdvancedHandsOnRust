use bevy::prelude::*;
use my_library_flappy_wrap::*;

#[derive(Component)]
struct Flappy;

#[derive(Component)]
struct Obstacle;

#[derive(Component)]
struct FlappyElement;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default, States)]
enum GamePhase {
  //START_HIGHLIGHT
  #[default] Loading,
  //END_HIGHLIGHT
  MainMenu,
  Flapping,
  GameOver
}

fn main() -> anyhow::Result<()> {
  let mut app = App::new();
  //START: Systems
  add_phase!(app, GamePhase, GamePhase::Flapping,
    start => [ setup ],
    run => [ flap, clamp, move_walls, hit_wall, cycle_animations,
      continual_parallax, physics_clock, sum_impulses, apply_gravity,
      //START_HIGHLIGHT
      apply_velocity, check_collisions::<Flappy, Obstacle>, rotate ],
      //END_HIGHLIGHT
    exit => [ cleanup::<FlappyElement> ]
  );
  //END: Systems

  app.add_event::<Impulse>();
  app.add_event::<PhysicsTick>();
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
    .add_plugins(
      AssetManager::new()
        .add_image("dragon", "flappy_dragon.png")?
        .add_image("wall", "wall.png")?
        .add_sound("flap", "dragonflap.ogg")?
        .add_sound("crash", "crash.ogg")?
        .add_sprite_sheet(
          "flappy",
          "flappy_sprite_sheet.png",
          62.0,
          65.0,
          4,
          1,
        )?
        .add_image("bg_static", "rocky-far-mountains.png")?
        .add_image("bg_far", "rocky-nowater-far.png")?
        .add_image("bg_mid", "rocky-nowater-mid.png")?
        .add_image("bg_close", "rocky-nowater-close.png")?,
    )
    .insert_resource(
      Animations::new()
        .with_animation(
          "Straight and Level",
          PerFrameAnimation::new(vec![
            AnimationFrame::new(2, 500, vec![AnimationOption::NextFrame]),
            AnimationFrame::new(
              3,
              500,
              vec![AnimationOption::GoToFrame(0)],
            ),
          ]),
        )
        .with_animation(
          "Flapping",
          PerFrameAnimation::new(vec![
            AnimationFrame::new(0, 66, vec![AnimationOption::NextFrame]),
            AnimationFrame::new(1, 66, vec![AnimationOption::NextFrame]),
            AnimationFrame::new(2, 66, vec![AnimationOption::NextFrame]),
            AnimationFrame::new(3, 66, vec![AnimationOption::NextFrame]),
            AnimationFrame::new(2, 66, vec![AnimationOption::NextFrame]),
            AnimationFrame::new(
              1,
              66,
              vec![AnimationOption::SwitchToAnimation(
                "Straight and Level".to_string(),
              )],
            ),
          ]),
        ),
    )
    .add_event::<OnCollision<Flappy, Obstacle>>()
    .run();

  Ok(())
}

fn setup(
  mut commands: Commands,
  mut rng: ResMut<RandomNumberGenerator>,
  assets: Res<AssetStore>,
  loaded_assets: Res<LoadedAssets>,
) {
  commands
    .spawn(Camera2d::default())
    .insert(FlappyElement);
  //START: FlappyPhysicsPosition
  spawn_animated_sprite!(
    assets, commands, "flappy", -490.0, 0.0, 10.0, "Straight and Level", 
    Flappy, FlappyElement,  Velocity::default(), ApplyGravity,
    AxisAlignedBoundingBox::new(62.0, 65.0),
    //START_HIGHLIGHT
    PhysicsPosition::new(Vec2::new(-490.0, 0.0))
    //END_HIGHLIGHT
  );
  //END: FlappyPhysicsPosition
  commands.insert_resource(StaticQuadTree::new(Vec2::new(1024.0, 768.0), 4));
  build_wall(&mut commands, &assets, rng.range(-5..5), &loaded_assets);
  
  spawn_image!(assets, commands, "bg_static", 0.0, 0.0, 1.0, &loaded_assets, FlappyElement);
  spawn_image!(
    assets,
    commands,
    "bg_far",
    0.0,
    0.0,
    2.0,
    &loaded_assets,
    FlappyElement,
    ContinualParallax::new(1280.0, 66, Vec2::new(1.0, 0.0))
  );
  spawn_image!(
    assets,
    commands,
    "bg_far",
    1280.0,
    0.0,
    2.0,
    &loaded_assets,
    FlappyElement,
    ContinualParallax::new(1280.0, 66, Vec2::new(1.0, 0.0))
  );
  spawn_image!(
    assets,
    commands,
    "bg_mid",
    0.0,
    0.0,
    3.0,
    &loaded_assets,
    FlappyElement,
    ContinualParallax::new(1280.0, 33, Vec2::new(1.0, 0.0))
  );
  spawn_image!(
    assets,
    commands,
    "bg_mid",
    1280.0,
    0.0,
    3.0,
    &loaded_assets,
    FlappyElement,
    ContinualParallax::new(1280.0, 33, Vec2::new(1.0, 0.0))
  );
  spawn_image!(
    assets,
    commands,
    "bg_close",
    0.0,
    0.0,
    4.0,
    &loaded_assets,
    FlappyElement,
    ContinualParallax::new(1280.0, 16, Vec2::new(2.0, 0.0))
  );
  spawn_image!(
    assets,
    commands,
    "bg_close",
    1280.0,
    0.0,
    4.0,
    &loaded_assets,
    FlappyElement,
    ContinualParallax::new(1280.0, 16, Vec2::new(2.0, 0.0))
  );
}

fn build_wall(commands: &mut Commands, assets: &AssetStore, gap_y: i32, loaded_assets: &LoadedAssets) {
  for y in -12..=12 {
    if y < gap_y - 4 || y > gap_y + 4 {
      //START: WallPhysicsPosition
      spawn_image!(
        assets,
        commands,
        "wall",
        512.0,
        y as f32 * 32.0,
        10.0,
        loaded_assets,
        Obstacle,
        FlappyElement,
        Velocity::new(-8.0, 0.0, 0.0),
        AxisAlignedBoundingBox::new(32.0, 32.0),
        //START_HIGHLIGHT
        PhysicsPosition::new(Vec2::new(512.0, y as f32 * 32.0))
        //END_HIGHLIGHT
      );
      //END: WallPhysicsPosition
    }
  }
}

fn flap(
  keyboard: Res<ButtonInput<KeyCode>>,
  mut query: Query<(Entity, &mut AnimationCycle)>,
  mut impulse: EventWriter<Impulse>,
) {
  if keyboard.pressed(KeyCode::Space) {
    if let Ok((flappy, mut animation)) = query.single_mut() {
      impulse.write(Impulse {
        target: flappy,
        amount: Vec3 {
          x: 0.0,
          y: 1.0,
          z: 0.0,
        },
        absolute: false,
      });
      animation.switch("Flapping");
    }
  }
}

//START: FlappyRotation
fn rotate(
  mut physics_position: Query<(&PhysicsPosition, &mut Transform), 
    With<Flappy>>,
) {
    physics_position.iter_mut().for_each(|(position, mut transform)| {
      if position.start_frame != position.end_frame {// <callout id="co.flappy.rotation.not_same" />
        let start = position.start_frame;// <callout id="co.flappy.rotation.start" />
        let end = position.end_frame;// <callout id="co.flappy.rotation.end" />
        let angle = end.angle_to(start) * 10.0;// <callout id="co.flappy.rotation.angle" />
        transform.rotation = Quat::from_rotation_z(angle);// <callout id="co.flappy.rotation.rotate" />
      }
    });
}
//END: FlappyRotation

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
  query: Query<&Transform, With<Obstacle>>,
  delete: Query<Entity, With<Obstacle>>,
  assets: Res<AssetStore>,
  mut rng: ResMut<RandomNumberGenerator>,
  loaded_assets: Res<LoadedAssets>,
) {
  let mut rebuild = false;
  for transform in query.iter() {
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
  mut collisions: EventReader<OnCollision<Flappy, Obstacle>>,
  mut state: ResMut<NextState<GamePhase>>,
  assets: Res<AssetStore>,
  loaded_assets: Res<LoadedAssets>,
  mut commands: Commands,
) {
  for _collision in collisions.read() {
    assets.play("crash", &mut commands, &loaded_assets);
    let _ = state.set(GamePhase::GameOver);
  }
}