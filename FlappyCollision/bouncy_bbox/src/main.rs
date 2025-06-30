use bevy::{prelude::*, diagnostic::{FrameTimeDiagnosticsPlugin, DiagnosticsStore}};
use my_library_flappy_collision::{*, egui::egui::Color32};
use my_library_flappy_collision::egui::EguiPrimaryContextPass;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default, States)]
pub enum GamePhase {
  #[default] Loading,
  MainMenu,
  Bouncing,
  GameOver,
}

#[derive(Component)]
struct BouncyElement;

#[derive(Resource, Default)]
struct CollisionTime{
  time: u128,
  checks: u32,
}

#[derive(Component)]
struct Ball;

//START: AABB
#[derive(Component)]
struct AxisAlignedBoundingBox {
  half_size: Vec2
}
//END: AABB

//START: implAABB
impl AxisAlignedBoundingBox {
  pub fn new(width: f32, height: f32) -> Self {
    Self {
      half_size: Vec2::new(width/2.0, height / 2.0)
    }
  }

  fn as_rect(&self, translate: Vec2) -> Rect2D {
    Rect2D::new(
      Vec2::new(translate.x - self.half_size.x, 
        translate.y - self.half_size.y),
      Vec2::new(translate.x + self.half_size.x, 
        translate.y + self.half_size.y),
    )
  }
}
//END: implAABB

//START: Rect2D
#[derive(Debug, Clone, Copy)]
struct Rect2D {
  min: Vec2,
  max: Vec2,
}
//END: Rect2D

//START: implRect2D
impl Rect2D {
  fn new(min: Vec2, max: Vec2) -> Self {
    Self { min, max }
  }

  fn intersect(&self, other: &Self) -> bool {
    self.min.x <= other.max.x
      && self.max.x >= other.min.x
      && self.min.y <= other.max.y
      && self.max.y >= other.min.y
  }
}
//END: implRect2D

fn main() -> anyhow::Result<()> {
  let mut app = App::new();
  add_phase!(app, GamePhase, GamePhase::Bouncing,
    start => [ setup ],
    run => [ warp_at_edge, collisions,
      continual_parallax, physics_clock, sum_impulses, apply_velocity ],
    exit => [ cleanup::<BouncyElement> ]
  );

  app.add_plugins(DefaultPlugins.set(WindowPlugin {
    primary_window: Some(Window {
      title: "Bounding Box Collision".to_string(),
      resolution: bevy::window::WindowResolution::new(
        1024.0, 768.0
      ),
      ..default()
    }),
    ..default()
  }))
      .add_plugins(FrameTimeDiagnosticsPlugin{
        ..default()
      })
      .add_event::<Impulse>()
      .add_event::<PhysicsTick>()
      .add_plugins(GameStatePlugin::new(
        GamePhase::MainMenu,
        GamePhase::Bouncing,
        GamePhase::GameOver))
      .add_plugins(RandomPlugin)
      .add_plugins(
        AssetManager::new().add_image("green_ball", "green_ball.png")?,
      )
      .add_systems(EguiPrimaryContextPass, show_performance.run_if(in_state(GamePhase::Bouncing)))
      .run();

  Ok(())
}

fn spawn_bouncies(
  to_spawn: usize,
  commands: &mut Commands,
  rng: &mut ResMut<RandomNumberGenerator>,
  assets: &AssetStore,
  loaded_assets: &LoadedAssets,
) {
  for _ in 0..to_spawn {
    let position =
      Vec3::new(rng.range(-512.0..512.0), rng.range(-384.0..384.0), 0.0);
    let velocity =
      Vec3::new(rng.range(-1.0..1.0), rng.range(-1.0..1.0), 0.0);
    //START: spawn_image
    spawn_image!(
      assets,
      commands,
      "green_ball",
      position.x,
      position.y,
      position.z,
      &loaded_assets,
      BouncyElement,
      Velocity::new(velocity.x, velocity.y, velocity.z),
      //START_HIGHLIGHT
      AxisAlignedBoundingBox::new(8.0, 8.0),
      //END_HIGHLIGHT
      Ball
    );
    //END: spawn_image
  }
}

fn setup(
  mut commands: Commands,
  mut rng: ResMut<RandomNumberGenerator>,
  assets: Res<AssetStore>,
  loaded_assets: Res<LoadedAssets>,
) {
  commands
    .spawn(Camera2d::default())
    .insert(BouncyElement);
  commands.insert_resource(CollisionTime::default());
  spawn_bouncies(1, &mut commands, &mut rng, &assets, &loaded_assets);
}

fn warp_at_edge(mut query: Query<&mut Transform, With<Ball>>) {
  for mut transform in query.iter_mut() {
    let pos = &mut transform.translation;
    if pos.x < -512.0 {
      pos.x = 512.0;
    } else if pos.x > 512.0 {
      pos.x = -512.0;
    }

    if pos.y < -384.0 {
      pos.y = 384.0;
    } else if pos.y > 384.0 {
      pos.y = -384.0;
    }
  }
}

fn show_performance(
  mut egui_context: egui::EguiContexts,
  diagnostics: Res<DiagnosticsStore>,
  collision_time: Res<CollisionTime>,
  mut commands: Commands,
  mut rng: ResMut<RandomNumberGenerator>,
  assets: Res<AssetStore>,
  query: Query<&Transform, With<Ball>>,
  loaded_assets: Res<LoadedAssets>,
) -> Result {
  let n_balls = query.iter().count();
  let fps = diagnostics
    .get(&FrameTimeDiagnosticsPlugin::FPS)
    .and_then(|fps| fps.average())
    .unwrap();
  egui::egui::Window::new("Performance").show(
    egui_context.ctx_mut()?,
    |ui| {
      let fps_text = format!("FPS: {fps:.1}");
      let color = match fps as u32 {
        0..=29 => Color32::RED,
        30..=59 => Color32::GOLD,
        _ => Color32::GREEN,
      };
      ui.colored_label(color, &fps_text);
      ui.colored_label(color, &format!("Collision Time: {} ms", collision_time.time));
      ui.label(&format!("Collision Checks: {}", collision_time.checks));
      ui.label(&format!("# Balls: {n_balls}"));
      if ui.button("Add Ball").clicked() {
        println!("{n_balls}, {}, {}", collision_time.time, collision_time.checks);
        spawn_bouncies(1, &mut commands, &mut rng, &assets, &loaded_assets);
      }
      if ui.button("Add 100 Balls").clicked() {
        println!("{n_balls}, {}, {}", collision_time.time, collision_time.checks);
        spawn_bouncies(100, &mut commands, &mut rng, &assets, &loaded_assets);
      }
      if ui.button("Add 1000 Balls").clicked() {
        println!("{n_balls}, {}, {}", collision_time.time, collision_time.checks);
        spawn_bouncies(1000, &mut commands, &mut rng, &assets, &loaded_assets);
      }
    },
  );
  Ok(())
}

fn bounce_on_collision(
  entity: Entity,
  ball_a: Vec3,
  ball_b: Vec3,
  impulse: &mut EventWriter<Impulse>,
) {
  let a_to_b = (ball_a - ball_b).normalize();
  impulse.write(Impulse {
    target: entity,
    amount: a_to_b / 8.0,
    absolute: false,
  });
}

//START: collisions
fn collisions(
  mut collision_time: ResMut<CollisionTime>,
  //START_HIGHLIGHT
  query: Query<(Entity, &Transform, &AxisAlignedBoundingBox)>,
  //END_HIGHLIGHT
  mut impulse: EventWriter<Impulse>
) {
  // Start the clock
  let now = std::time::Instant::now();

  // Bounding Box Naïve Collision
  let mut n = 0;
  for (entity_a, ball_a, box_a) in query.iter() {// <callout id="bbox.iter_a" />
    let box_a = box_a.as_rect(ball_a.translation.truncate());// <callout id="bbox.truncate" />
    for (entity_b, ball_b, box_b) in query.iter() {// <callout id="bbox.iter_b" />
      if entity_a != entity_b {// <callout id="bbox.not_same" />
        let box_b = box_b.as_rect(ball_b.translation.truncate());// <callout id="bbox.box_b" />
        if box_a.intersect(&box_b) {
          bounce_on_collision(entity_a, ball_a.translation, 
            ball_b.translation, &mut impulse);// <callout id="bbox.collide" />
        }
        n += 1;
      }
    }
  }

  // Store the time result
  collision_time.time = now.elapsed().as_millis();
  collision_time.checks = n;
}
//END: collisions