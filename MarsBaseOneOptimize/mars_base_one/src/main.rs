use std::sync::atomic::AtomicBool;
use bevy::render::mesh::PrimitiveTopology;
use bevy::prelude::*;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use my_library::*;
use my_library::egui::egui::Color32;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default, States)]
enum GamePhase {
  #[default]
  Loading,
  MainMenu,
  //START_HIGHLIGHT
  WorldBuilding,
  //END_HIGHLIGHT
  Playing,
  GameOver,
}

#[derive(Component)]
struct GameElement;

#[derive(Component)]
struct Player;

fn main() -> anyhow::Result<()> {
  let mut app = App::new();
  add_phase!(app, GamePhase, GamePhase::WorldBuilding,
    start => [ spawn_builder ],
    run => [ show_builder ],
    exit => [ ]
  );
  add_phase!(app, GamePhase, GamePhase::Playing,
    start => [ setup ],
    run => [ movement, end_game, physics_clock, sum_impulses, apply_gravity, apply_velocity,
      terminal_velocity.after(apply_velocity), check_collisions::<Player, Ground>, bounce, 
      camera_follow.after(terminal_velocity),
      show_performance],
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
      .add_plugins(FrameTimeDiagnosticsPlugin{
        ..default()
      })
      .add_plugins(RandomPlugin)
      .add_plugins(GameStatePlugin::new(
        GamePhase::MainMenu,
        GamePhase::WorldBuilding,
        GamePhase::GameOver,
      ))
      .add_plugins(
        AssetManager::new().add_image("ship", "ship.png")?
            .add_image("ground", "ground.png")?
      )
      .insert_resource(Animations::new())
      .add_event::<OnCollision<Player, Ground>>()
      .run();

  Ok(())
}

static WORLD_READY: AtomicBool = AtomicBool::new(false);
use std::sync::Mutex;
use bevy::asset::RenderAssetUsages;
use bevy::render::camera::ScalingMode;

static NEW_WORLD: Mutex<Option<World>> = Mutex::new(None);

fn spawn_builder() {
  use std::sync::atomic::Ordering;
  // Clear the build state
  WORLD_READY.store(false, Ordering::Relaxed);

  // Spawn a "building world" message

  //Start a world building thread
  std::thread::spawn(|| {
    // Make our own random number generator
    let mut rng = my_library::RandomNumberGenerator::new();
    // Spawn the world
    let world = World::new(200, 200, &mut rng);

    // Store the world
    let mut lock = NEW_WORLD.lock().unwrap();
    *lock = Some(world);

    // Notify of success
    WORLD_READY.store(true, Ordering::Relaxed);
  });
}

fn show_builder(
  mut state: ResMut<NextState<GamePhase>>,
  mut egui_context: egui::EguiContexts,
) {
  egui::egui::Window::new("Performance").show(
    egui_context.ctx_mut(),
    |ui| {
      ui.label("Building World");
    });

  if WORLD_READY.load(std::sync::atomic::Ordering::Relaxed) {
    state.set(GamePhase::Playing);
  }
}
//END: WorldBuildingDone

fn setup(
  mut commands: Commands,
  assets: Res<AssetStore>,
  loaded_assets: Res<LoadedAssets>,
  mut meshes: ResMut<Assets<Mesh>>,
  mut materials: ResMut<Assets<ColorMaterial>>,
) {
  let cb = Camera2d::default();
  let projection = Projection::Orthographic(OrthographicProjection {
    scaling_mode: ScalingMode::WindowSize,
    scale: 0.5,
    ..OrthographicProjection::default_2d()
  });

  commands
      .spawn(cb).insert(projection)
      .insert(GameElement)
      .insert(MyCamera);

  spawn_image!(
    assets,
    commands,
    "ship",
    0.0,
    200.0,
    1.0,
    &loaded_assets,
    GameElement,
    Player,
    Velocity::default(),
    PhysicsPosition::new(Vec2::new(0.0, 200.0)),
    ApplyGravity,
    AxisAlignedBoundingBox::new(24.0, 24.0)
  );

  //let world = World::new(200, 200, &mut rng);
  let mut lock = NEW_WORLD.lock().unwrap();
  let world = lock.take().unwrap();
  world.spawn(&assets, &mut commands, &loaded_assets, &mut meshes, &mut materials);
  commands.insert_resource(StaticQuadTree::new(Vec2::new(400.0 * 24.0, 400.0 * 24.0), 6));
}

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
      amount: transform.local_y() .as_vec3(),// <callout id="mb1.transform" />
      absolute: false,
      source: 1,
    });
  }
}

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
    //state.set(GamePhase::GameOver);
  }
}

fn camera_follow(
  player_query: Query<&Transform, (With<Player>, Without<MyCamera>)>,
  mut camera_query: Query<&mut Transform, (With<MyCamera>, Without<Player>)>,
) {
  let Ok(player) = player_query.single() else {
    return;
  };
  let Ok(mut camera) = camera_query.single_mut() else {
    return;
  };
  camera.translation = Vec3::new(player.translation.x, player.translation.y, 10.0);
}

fn bounce(
  mut collisions: EventReader<OnCollision<Player, Ground>>,
  mut player_query: Query<&PhysicsPosition, With<Player>>,
  ground_query: Query<&PhysicsPosition, With<Ground>>,
  mut impulses: EventWriter<Impulse>,
) {
  let mut bounce = Vec2::default();
  let mut entity = None;
  let mut bounces = 0;
  for collision in collisions.read() {
    if let Ok(player) = player_query.single_mut() {
      if let Ok(ground) = ground_query.get(collision.entity_b) {
        entity = Some(collision.entity_a);
        let difference = player.start_frame - ground.start_frame;
        bounces += 1;
        bounce = difference;
      }
    }
  }
  if bounce != Vec2::default() {
    //println!("Bounce: {:?}", bounce);
    bounce = bounce.normalize();
    impulses.write(Impulse {
      target: entity.unwrap(),
      amount: Vec3::new(bounce.x / bounces as f32, bounce.y / bounces as f32, 0.0),
      absolute: true,
      source: 2,
    });
  }
}

#[derive(Component)]
struct Ground;

#[derive(Component)]
struct MyCamera;

//START: WorldWithMesh
struct World {
  solid: Vec<bool>,
  width: usize,
  height: usize,
  //START_HIGHLIGHT
  mesh: Option<Mesh>,
  tile_positions: Vec<(f32, f32)>,
  //END_HIGHLIGHT
}
//END: WorldWithMesh

impl World {
  fn mapidx(&self, x: usize, y: usize) -> usize {
    y * self.width + x
  }

  fn find_random_closed_tile(&self, rng: &mut RandomNumberGenerator) -> (usize, usize) {
    loop {
      let x = rng.range(0 .. self.width);
      let y = rng.range(0 .. self.height);
      let idx = self.mapidx(x, y);
      if self.solid[idx] {
        return (x, y);
      }
    }
  }

  fn clear_tiles(&mut self, x: usize, y: usize) {
    for offset_x in -1 ..= 1 {
      for offset_y in -1 ..= 1 {
        let x = x as isize + offset_x;
        let y = y as isize + offset_y;
        if x > 0 && x < self.width as isize -1 && y > 0 && y < self.height as isize {
          let idx = self.mapidx(x as usize, y as usize);
          self.solid[idx] = false;
        }
      }
    }
  }

  fn clear_line(&mut self, start: (usize, usize), end: (usize, usize)) {
    let (mut x, mut y) = (start.0 as f32, start.1 as f32);
    let (slope_x, slope_y) = (
      (end.0 as f32 - x) / self.width as f32,
      (end.1 as f32 - y) / self.height as f32,
    );
    loop {
      let (tx, ty) = (x as usize, y as usize);
      if tx < 1 || tx > self.width-1 || ty < 1 || ty > self.height-1 {
        break;
      }
      if tx == end.0 && ty == end.1 {
        break;
      }
      self.clear_tiles(x as usize, y as usize);
      x += slope_x;
      y += slope_y;
    }
  }

  //START: WorldNew
  fn new(
    width: usize, height: usize, 
    rng: &mut RandomNumberGenerator
  ) -> Self {
    let mut result = Self {
      width,
      height,
      solid: vec![true; width * height],
      //START_HIGHLIGHT
      mesh: None,
      //END_HIGHLIGHT
      tile_positions: Vec::new(),
    };
    //END: WorldNew

    // Set the center tile and surrounding tiles to be empty
    result.clear_tiles(width / 2, height / 2);
    let mut holes = vec![(width / 2, height / 2)];

    // Blast some holes in the center
    for _ in 0 .. 10 {
      let x = rng.range(5 .. width-5);
      let y = rng.range(5 .. height-5);
      holes.push((x, y));
      result.clear_tiles(x, y);
      result.clear_tiles(x+2, y);
      result.clear_tiles(x-2, y);
      result.clear_tiles(x, y+2);
      result.clear_tiles(x, y-2);
    }

    // Cut a line between each hole
    for i in 0 .. holes.len() {
      let start = holes[i];
      let end = holes[(i + 1) % holes.len()];
      result.clear_line(start, end);
    }

    // Carve a borehole
    for y in height/2 .. height {
      result.clear_tiles(width / 2, y);
    }

    // Outward diffusion
    let mut done = false;
    while !done {
      let start_tile = holes[rng.range(0..10)];
      let target = result.find_random_closed_tile(rng);
      let (mut x, mut y) = (start_tile.0 as f32, start_tile.1 as f32);
      let (slope_x, slope_y) = (
        (target.0 as f32 - x) / width as f32,
        (target.1 as f32 - y) / height as f32,
      );

      loop {
        if x < 1.0 || x > width as f32 || y < 1.0 || y > height as f32 {
          break;
        }
        let tile_id = result.mapidx(x as usize, y as usize);
        if result.solid[tile_id] {
          result.clear_tiles(x as usize, y as usize);
          break;
        }
        x += slope_x;
        y += slope_y;
      }

      let solid_count = result.solid.iter().filter(|s| **s).count();
      let solid_percent = solid_count as f32 / (width * height) as f32;
      if solid_percent < 0.5 { done = true; }
    }

    let (mesh, tile_positions) = result.build_mesh();
    result.mesh = Some(mesh);
    result.tile_positions = tile_positions;

    result
  }

  //START: BuildMesh
  fn build_mesh(&self) -> (Mesh, Vec<(f32, f32)>) {
    let mut position = Vec::new();
    let mut uv = Vec::new();
    let mut tile_positions = Vec::new();//<callout id="mbo.mesh.vecs" />
    let x_offset = (self.width as f32 / 2.0) * 24.0;
    let y_offset =(self.height as f32) * 24.0;//<callout id="mbo.mesh.offsets" />
    for y in 0 .. self.height {
      for x in 0 .. self.width {
        if self.solid[self.mapidx(x, y)] {

          let left = (x as f32 * 24.0) - x_offset;//<callout id="mbo.mesh.positions" />
          let right = ((x as f32 + 1.0) * 24.0) - x_offset;
          let top = (y as f32 * 24.0) - y_offset;
          let bottom = ((y as f32 + 1.0) * 24.0) - y_offset;

          position.push([left, bottom, 1.0]);//<callout id="mbo.mesh.triangles" />
          position.push([right, bottom, 1.0]);
          position.push([right, top, 1.0]);
          position.push([right, top, 1.0]);
          position.push([left, bottom, 1.0]);
          position.push([left, top, 1.0]);

          uv.push([0.0, 1.0]);//<callout id="mbo.mesh.uvs" />
          uv.push([1.0, 1.0]);
          uv.push([1.0, 0.0]);
          uv.push([1.0, 0.0]);
          uv.push([0.0, 1.0]);
          uv.push([0.0, 0.0]);

          tile_positions.push((left + 12.0, top + 12.0));//<callout id="mbo.mesh.push" />
        }
      }
    }

    (
      Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, position)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv),

      tile_positions
    )
  }
  //END: BuildMesh

  //START: SpawnWithMesh
  fn spawn(
    &self, 
    assets: &AssetStore, 
    commands: &mut Commands, 
    loaded_assets: &LoadedAssets,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
  ) {
    let mesh = self.mesh.as_ref().unwrap().clone();
    let mesh_handle = meshes.add(mesh);
    let material_handle = materials.add(ColorMaterial {
      texture: Some(assets.get_handle("ground", loaded_assets).unwrap()),
      ..Default::default()
    });
    commands.spawn(Mesh2d(mesh_handle ))
        .insert(MeshMaterial2d(material_handle ))
        .insert(Transform::from_xyz(0.0, 0.0, 0.0));

    for (x, y) in self.tile_positions.iter() {
      commands.spawn_empty()
        .insert(GameElement)
        .insert(Ground)
        .insert(PhysicsPosition::new(Vec2::new(*x, *y)))
        .insert(AxisAlignedBoundingBox::new(24.0, 24.0));
    }
  }
  //END: SpawnWithMesh
}

//START: ShowFPS
fn show_performance(
  diagnostics: Res<DiagnosticsStore>,
  mut egui_context: egui::EguiContexts,
) {
  let fps = diagnostics
      .get(&FrameTimeDiagnosticsPlugin::FPS)
      .and_then(|fps| fps.average())
      .unwrap_or(0.0);
  egui::egui::Window::new("Performance").show(
    egui_context.ctx_mut(),
    |ui| {
      let fps_text = format!("FPS: {fps:.1}");
      let color = match fps as u32 {
        0..=29 => Color32::RED,
        30..=50 => Color32::GOLD,
        _ => Color32::GREEN,
      };
      ui.colored_label(color, &fps_text);
    });
}
//END: ShowFPS