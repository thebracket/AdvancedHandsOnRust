use bevy::{
  diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
  prelude::*,
};
use my_library_flappy_collision::{egui::egui::Color32, *};
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

#[derive(Component)]
struct Ball;

#[derive(Resource, Default)]
struct CollisionTime {
  time: u128,
  checks: u32,
  fps: f64,
}

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
      title: "Naieve Collision".to_string(),
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
      Ball
    );
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

//START: show_performance
fn show_performance(
  mut egui_context: egui::EguiContexts,
  diagnostics: Res<DiagnosticsStore>,//<callout id="bouncy.diagnostics" />
  mut collision_time: ResMut<CollisionTime>,
  mut commands: Commands,
  mut rng: ResMut<RandomNumberGenerator>,
  assets: Res<AssetStore>,
  query: Query<&Transform, With<Ball>>,
  loaded_assets: Res<LoadedAssets>,
) -> Result {
  let n_balls = query.iter().count();//<callout id="bouncy.count_balls" />
  let fps = diagnostics//<callout id="bouncy.get_fps" />
    .get(&FrameTimeDiagnosticsPlugin::FPS)
    .and_then(|fps| fps.average())
    .unwrap();
  collision_time.fps = fps;
  egui::egui::Window::new("Performance").show(
    egui_context.ctx_mut()?,
    |ui| {
      let fps_text = format!("FPS: {fps:.1}");//<callout id="bouncy.format_ball_count" />
      let color = match fps as u32 {//<callout id="bouncy.colorize" />
        0..=29 => Color32::RED,
        30..=59 => Color32::GOLD,
        _ => Color32::GREEN,
      };
      ui.colored_label(color, &fps_text);
      ui.colored_label(
        color,
        &format!("Collision Time: {} ms", collision_time.time),
      );
      ui.label(&format!("Collision Checks: {}", collision_time.checks));
      ui.label(&format!("# Balls: {n_balls}"));
      if ui.button("Add Ball").clicked() {//<callout id="bouncy.add_balls" />
        println!(
          "{n_balls}, {}, {}, {:.0}",
          collision_time.time, collision_time.checks, collision_time.fps
        );
        spawn_bouncies(1, &mut commands, &mut rng, &assets, 
          &loaded_assets);
      }
      if ui.button("Add 100 Balls").clicked() {
        println!(
          "{n_balls}, {}, {}, {:.0}",
          collision_time.time, collision_time.checks, collision_time.fps
        );
        spawn_bouncies(100, &mut commands, &mut rng, &assets, 
          &loaded_assets);
      }
      if ui.button("Add 1000 Balls").clicked() {
        println!(
          "{n_balls}, {}, {}, {:.0}",
          collision_time.time, collision_time.checks, collision_time.fps
        );
        spawn_bouncies(1000, &mut commands, &mut rng, &assets, 
          &loaded_assets);
      }
    },
  );
  Ok(())
}
//END: show_performance

//START: bounce_on_collision
fn bounce_on_collision(
  entity: Entity,
  ball_a: Vec3,
  ball_b: Vec3,
  impulse: &mut EventWriter<Impulse>,
) {
  let a_to_b = (ball_a - ball_b).normalize();//<callout id="bouncy.normalize" />
  impulse.write(Impulse {
    target: entity,
    amount: a_to_b / 8.0,//<callout id="bouncy.div_dist" />
    absolute: false,
  });
}
//END: bounce_on_collision

//START: collisions
fn collisions(
  mut collision_time: ResMut<CollisionTime>,
  query: Query<(Entity, &Transform), With<Ball>>,
  mut impulse: EventWriter<Impulse>,
) {
  // Start the clock
  let now = std::time::Instant::now();

  // Naïve Collision
  let mut n = 0;
  for (entity_a, ball_a) in query.iter() {
    query
      .iter()
      .filter(|(entity_b, _)| *entity_b != entity_a)
      .filter(|(_, ball_b)| {
        n += 1; // Count the collision check
        ball_a.translation.distance(ball_b.translation) < 8.0
      })
      .for_each(|(_, ball_b)| {
        bounce_on_collision(
          entity_a,
          ball_a.translation,
          ball_b.translation,
          &mut impulse,
        );
      });
  }

  // Store the time result
  collision_time.time = now.elapsed().as_millis();
  collision_time.checks = n;
}
//END: collisions