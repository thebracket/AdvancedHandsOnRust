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
  WorldBuilding,
  Playing,
  GameOver,
}

#[derive(Component)]
struct GameElement;

//START: Player
#[derive(Component)]
struct Player {
  miners_saved: u32,
  shields: i32,
  fuel: i32,
}
//END: Player

fn main() -> anyhow::Result<()> {
  let mut app = App::new();
  add_phase!(app, GamePhase, GamePhase::WorldBuilding,
    start => [ spawn_builder ],
    run => [ show_builder ],
    exit => [ ]
  );
  //START: FinalizeSystems
  add_phase!(app, GamePhase, GamePhase::Playing,
    start => [ setup ],
    run => [ movement, end_game, physics_clock, sum_impulses, apply_gravity, 
        apply_velocity,
      terminal_velocity, check_collisions::<Player, Ground>, bounce, 
        camera_follow,
      show_performance, spawn_particle_system, particle_age_system,
      miner_beacon,
      score_display, check_collisions::<Player, Miner>,
      check_collisions::<Player, Fuel>, check_collisions::<Player, Battery>,
      collect_game_element_and_despawn::<Miner, { BurstColor::Green as u8 }>,
      collect_game_element_and_despawn::<Fuel, { BurstColor::Orange as u8 }>,
      collect_game_element_and_despawn::<Battery, { BurstColor::Magenta 
        as u8 }>
      ], 
    exit => [ cleanup::<GameElement> ]
  );
  //END: FinalizeSystems

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
      .add_plugins(FrameTimeDiagnosticsPlugin::default())
      .add_plugins(RandomPlugin)
      .add_plugins(GameStatePlugin::new(
        GamePhase::MainMenu,
        GamePhase::WorldBuilding,
        GamePhase::GameOver,
      ))
      .add_plugins(
        //START: Assets
        AssetManager::new().add_image("ship", "ship.png")?
            .add_image("ground", "ground.png")?
            .add_image("backdrop", "backing.png")?
            .add_image("particle", "particle.png")?
            .add_image("mothership", "mothership.png")?
            //START_HIGHLIGHT
            .add_image("spaceman", "spaceman.png")?
            .add_image("fuel", "fuel.png")?
            .add_image("battery", "battery.png")?
            //END_HIGHLIGHT
        //END: Assets

      )
      .insert_resource(Animations::new())
      .add_event::<OnCollision<Player, Ground>>()
      .add_event::<OnCollision<Player, Miner>>()
      .add_event::<OnCollision<Player, Fuel>>()
      .add_event::<OnCollision<Player, Battery>>()
      .add_event::<SpawnParticle>()
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

  //START: WorldShuffle
  //Start a world building thread
  std::thread::spawn(|| {
    // Make our own random number generator
    let mut rng = my_library::RandomNumberGenerator::new();
    // Spawn the world
    let mut world = World::new(200, 200, &mut rng);

    // Shuffle possible miner positions and limit the size to 20
    //START_HIGHLIGHT
    use my_library::rand::seq::SliceRandom;
    world.spawn_positions.shuffle(&mut rng.rng);
    //END_HIGHLIGHT
    //END: WorldShuffle

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
    10.0,
    &loaded_assets,
    GameElement,
    Player { miners_saved: 0, shields: 500, fuel: 100_000 },
    Velocity::default(),
    PhysicsPosition::new(Vec2::new(0.0, 200.0)),
    ApplyGravity,
    AxisAlignedBoundingBox::new(24.0, 24.0)
  );

  //START: SpawnMothership
  spawn_image!(
    assets,
    commands,
    "mothership",
    0.0,
    400.0,
    10.0,
    &loaded_assets,
    GameElement
  );
  //END: SpawnMothership

  //let world = World::new(200, 200, &mut rng);
  let mut lock = NEW_WORLD.lock().unwrap();
  let world = lock.take().unwrap();
  world.spawn(&assets, &mut commands, &loaded_assets, &mut meshes, &mut materials);
  commands.insert_resource(StaticQuadTree::new(Vec2::new(400.0 * 24.0, 400.0 * 24.0), 6));

  //START: Backdrop
  // Backdrop
  let x = 100.0;
  let y = 100.0;
  let x_scale = (200.0 * 24.0) / 1720.0;// <callout id="mbo.backdrop.x_scale" />
  let y_scale = (300.0 * 24.0) / 1024.0;// <callout id="mbo.backdrop.y_scale" />
  let center_x = (x as f32 * 24.0) - ((200.0 / 2.0) * 24.0);
  let center_y = ((y as f32 + 1.0) * 24.0) - (200.0 * 24.0);
  let mut transform = Transform::from_xyz(center_x, center_y, -10.0);// <callout id="mbo.backdrop.transform" />
  transform.scale = Vec3::new(x_scale, y_scale, 1.0);// <callout id="mbo.backdrop.scale" />
  commands
      .spawn(Sprite::from_image(assets.get_handle("backdrop", &loaded_assets).unwrap()))
      .insert(transform)
      .insert(GameElement);// <callout id="mbo.backdrop.insert" />
  //END: Backdrop

}

//START: Movement
fn movement(
  keyboard: Res<ButtonInput<KeyCode>>,
  mut player_query: Query<(Entity, &mut Transform, &mut Player)>,
  mut impulses: EventWriter<Impulse>,
  mut particles: EventWriter<SpawnParticle>,
) {
  let Ok((entity, mut transform, mut player)) = player_query.single_mut()
    else {
      return;
    };
  if keyboard.pressed(KeyCode::ArrowLeft) {
    transform.rotate(Quat::from_rotation_z(f32::to_radians(2.0)));

    //START_HIGHLIGHT
    particles.write(SpawnParticle{
      position: -transform.local_x().truncate() + Vec2::new(
        transform.translation.x, transform.translation.y),
      color: LinearRgba::new(0.0, 1.0, 1.0, 1.0),
      velocity: transform.local_x().as_vec3(),
    });
    //END_HIGHLIGHT
  }
  if keyboard.pressed(KeyCode::ArrowRight) {
    transform.rotate(Quat::from_rotation_z(f32::to_radians(-2.0)));

    //START_HIGHLIGHT
    particles.write(SpawnParticle{
      position: transform.local_x().truncate() + Vec2::new(
        transform.translation.x, transform.translation.y),
      color: LinearRgba::new(0.0, 1.0, 1.0, 1.0),
      velocity: -transform.local_x().as_vec3(),
    });
    //END_HIGHLIGHT
  }
  if keyboard.pressed(KeyCode::ArrowUp) {
    if player.fuel > 0 {
      impulses.write(Impulse {
        target: entity,
        amount: transform.local_y().as_vec3(),
        absolute: false,
      });
      particles.write(SpawnParticle{
        position: transform.local_y().truncate() + Vec2::new(
          transform.translation.x, transform.translation.y),
        color: LinearRgba::new(0.0, 1.0, 1.0, 1.0),
        velocity: -transform.local_y().as_vec3(),
      });
      player.fuel -= 1;
    }
  }
}
//END: Movement

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

//START: EndGame
fn end_game(
  mut state: ResMut<NextState<GamePhase>>,
  player_query: Query<&Player>,
) {
  let Ok(player) = player_query.single() else {
    return;
  };
  if player.miners_saved == 20 {
    // You won!
    state.set(GamePhase::GameOver);
  }
}
//END: EndGame

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

//START: Bounce
fn bounce(
  mut collisions: EventReader<OnCollision<Player, Ground>>,
  mut player_query: Query<(&PhysicsPosition, &mut Player)>,
  ground_query: Query<&PhysicsPosition, With<Ground>>,
  mut impulses: EventWriter<Impulse>,
  mut particles: EventWriter<SpawnParticle>,
  mut state: ResMut<NextState<GamePhase>>,
) {
  let mut bounce = Vec2::default();
  let mut entity = None;
  let mut bounces = 0;
  // Re-using the bounce techniques from the collisions chapter.
  for collision in collisions.read() {
    if let Ok((player, _)) = player_query.single_mut() {
      if let Ok(ground) = ground_query.get(collision.entity_b) {
        entity = Some(collision.entity_a);
        let difference = player.start_frame - ground.start_frame;
        bounces += 1;
        bounce = difference;
      }
    }
  }
  if bounce != Vec2::default() {
    bounce = bounce.normalize();
    impulses.write(Impulse {
      target: entity.unwrap(),
      amount: Vec3::new(bounce.x / bounces as f32, bounce.y / bounces as f32, 0.0),
      absolute: true,
    });

    // Spawn a burst of particles
    let Ok((player_pos, mut player)) = player_query.single_mut()
      else {
        return;
      };
    particle_burst(
      player_pos.end_frame,
      LinearRgba::new(0.0, 0.0, 1.0, 1.0),
      &mut particles,
      3.0,
    );
    // Reduce the player's shield level
    player.shields -= 1;
    if player.shields <= 0 {
      state.set(GamePhase::GameOver);
    }
  }
}
//END: Bounce

#[derive(Component)]
struct Ground;

#[derive(Component)]
struct MyCamera;

//START: WorldWithMesh
struct World {
  solid: Vec<bool>,
  width: usize,
  height: usize,
  mesh: Option<Mesh>,
  tile_positions: Vec<(f32, f32)>,
  //START_HIGHLIGHT
  spawn_positions: Vec<(f32, f32)>,
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

  fn new(width: usize, height: usize, rng: &mut RandomNumberGenerator) -> Self {
    let mut result = Self {
      width,
      height,
      solid: vec![true; width * height],
      mesh: None,
      tile_positions: Vec::new(),
      spawn_positions: Vec::new(),
    };

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
      if solid_percent < 0.7 { done = true; }
    }

    let (mesh, tile_positions, possible_miner_positions) = result.build_mesh();
    result.mesh = Some(mesh);
    result.tile_positions = tile_positions;
    result.spawn_positions = possible_miner_positions;

    result
  }

  fn build_mesh(&self) -> (Mesh, Vec<(f32, f32)>, Vec<(f32, f32)>) {
    let mut position = Vec::new();
    let mut uv = Vec::new();
    let mut tile_positions = Vec::new();
    let mut possible_miner_positions = Vec::new();
    for y in 0 .. self.height {
      for x in 0 .. self.width {

        let left = (x as f32 * 24.0) - ((self.width as f32 / 2.0) * 24.0);
        let right = ((x as f32 + 1.0) * 24.0) - ((self.width as f32 / 2.0) * 24.0);
        let top = (y as f32 * 24.0) - ((self.height as f32) * 24.0);
        let bottom = ((y as f32 + 1.0) * 24.0) - ((self.height as f32) * 24.0);


        if self.solid[self.mapidx(x, y)] {

          position.push([left, bottom, 10.0]);
          position.push([right, bottom, 10.0]);
          position.push([right, top, 10.0]);
          position.push([right, top, 10.0]);
          position.push([left, bottom, 10.0]);
          position.push([left, top, 10.0]);

          uv.push([0.0, 0.0]);
          uv.push([1.0, 0.0]);
          uv.push([1.0, 1.0]);
          uv.push([1.0, 1.0]);
          uv.push([0.0, 0.0]);
          uv.push([0.0, 1.0]);

          // Only push the physics position if the tile is on the edge,
          // and the tile isn't completely surrounded by solid tiles.
          let mut needs_physics = false;

          if x==0 || x > self.width-3 || y==0 || y > self.height-3 {
            // On the edge
            needs_physics = true;
          } else {
            // Are we surrounded by solid tiles?
            let solid_count =
                self.solid[self.mapidx(x-1, y)] as u8
              + self.solid[self.mapidx(x+1, y)] as u8
              + self.solid[self.mapidx(x, y+1)] as u8
              + self.solid[self.mapidx(x, y+1)] as u8;
            if solid_count < 4 {
              needs_physics = true;
            }
          }
          
          if  needs_physics {
            tile_positions.push((left + 12.0, top + 12.0));
          }
          //START: PossibleMiner
        } 
        //START_HIGHLIGHT
        else {
          if x > 1 && x < self.width-3 && y > 1 && y < self.height-3 &&
          self.solid[self.mapidx(x, y-1)] {
            possible_miner_positions.push((left + 12.0, top + 12.0));
          }
        }
        //END_HIGHLIGHT
      }
    }

    (
      Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, position)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv),

      tile_positions, 
      //START_HIGHLIGHT
      possible_miner_positions
      //END_HIGHLIGHT
    )
  }
        //END: PossibleMiner

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

    //START: SpawnMiners
    // Spawn miners
    for (x, y) in self.spawn_positions.iter().take(20) {//<callout id="elements.spawn_miners.iter" />
      spawn_image!(//<callout id="elements.spawn_miners.spawn_image" />
        assets,
        commands,
        "spaceman",
        *x,//<callout id="elements.spawn_miners.deref" />
        *y,
        10.0,
        loaded_assets,
        GameElement,
        Miner,
        Velocity::default(),
        PhysicsPosition::new(Vec2::new(*x, *y)),
        // Extra Large Hitbox
        AxisAlignedBoundingBox::new(48.0, 48.0)
      );
    }
    //END: SpawnMiners

    //START: SpawnFuel
    // Spawn fuel
    for (x, y) in self.spawn_positions.iter().skip(20).take(20) {//<callout id="elements.spawn_fuel.iter_take" />
      spawn_image!(
        assets,
        commands,
        "fuel",
        *x,
        *y,
        10.0,
        loaded_assets,
        GameElement,
        Fuel,
        Velocity::default(),
        PhysicsPosition::new(Vec2::new(*x, *y)),
        // Extra Large Hitbox
        AxisAlignedBoundingBox::new(48.0, 48.0)
      );
    }
    //END: SpawnFuel

    //START: SpawnBatteries
    // Spawn batteries
    for (x, y) in self.spawn_positions.iter().skip(40).take(10) {
      spawn_image!(
        assets,
        commands,
        "battery",
        *x,
        *y,
        10.0,
        loaded_assets,
        GameElement,
        Battery,
        Velocity::default(),
        PhysicsPosition::new(Vec2::new(*x, *y)),
        // Extra Large Hitbox
        AxisAlignedBoundingBox::new(48.0, 48.0)
      );
      //END: SpawnBatteries
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

//START: Particle
#[derive(Component)]
pub struct Particle {
  pub lifetime: f32,
}
//END: Particle

//START: ParticleAge
fn particle_age_system(
  time: Res<Time>,
  mut commands: Commands,
  mut query: Query<(Entity, &mut Particle, &mut Sprite)>,
) {
  for (entity, mut particle, mut sprite) in 
    query.iter_mut() 
  {
    particle.lifetime -= time.delta_secs();
    if particle.lifetime <= 0.0 {
      commands.entity(entity).despawn();
    }

    // Adjust the color
    sprite.color.set_alpha(particle.lifetime / 2.0);
  }
}
//END: ParticleAge

//START: SpawnParticle
#[derive(Event)]
pub struct SpawnParticle{
  position: Vec2,
  color: LinearRgba,
  velocity: Vec3,
}
//END: SpawnParticle

//START: SpawnParticleSystem
// Receive messages to spawn particles
fn spawn_particle_system(
  mut commands: Commands,
  mut reader: EventReader<SpawnParticle>,
  assets: Res<AssetStore>,
  loaded_assets: Res<LoadedAssets>,
) {
  for particle in reader.read() {
    let mut sprite = Sprite::from_image(
      assets.get_handle("particle", &loaded_assets).unwrap());
    sprite.color = particle.color.into();
    commands
        .spawn(sprite)
        .insert(Transform::from_xyz(
          particle.position.x, particle.position.y, 5.0))
        .insert(GameElement)
        .insert(Particle { lifetime: 2.0 })
        .insert(Velocity(particle.velocity))
        .insert(PhysicsPosition::new(particle.position));
  }
}
//END: SpawnParticleSystem

//START: OnCollect
trait OnCollect {
  fn effect(player: &mut Player);
}
//END: OnCollect

//START: OnCollectImpl
impl OnCollect for Miner {
  fn effect(player: &mut Player) {
    player.miners_saved += 1;
  }
}

impl OnCollect for Fuel {
  fn effect(player: &mut Player) {
    player.fuel += 1000;
  }
}

impl OnCollect for Battery {
  fn effect(player: &mut Player) {
    player.shields += 100;
  }
}
//END: OnCollectImpl

//START: BurstColor
#[repr(u8)]
enum BurstColor {
  Green,
  Orange,
  Magenta,
}
//END: BurstColor

//START: BurstColorFromU8
impl From<u8> for BurstColor {
  fn from(value: u8) -> Self {
    match value {
      0 => BurstColor::Green,
      1 => BurstColor::Orange,
      2 => BurstColor::Magenta,
      _ => panic!("Invalid BurstColor value"),
    }
  }
}
//END: BurstColorFromU8

//START: BurstColorIntoVec3
impl Into<LinearRgba> for BurstColor {
  fn into(self) -> LinearRgba {
    match self {
      BurstColor::Green => LinearRgba::new(0.0, 1.0, 0.0, 1.0),
      BurstColor::Orange => LinearRgba::new(1.0, 0.5, 0.0, 1.0),
      BurstColor::Magenta => LinearRgba::new(1.0, 0.0, 1.0, 1.0),
    }
  }
}
//END: BurstColorIntoVec3


//START: CollectGameElement
fn collect_game_element_and_despawn<T:Component + OnCollect, const COLOR: u8>
(
  mut collisions: EventReader<OnCollision<Player, T>>,// <callout id="elements.on_collision_generic" />
  mut commands: Commands,
  mut player: Query<(&mut Player, &Transform)>,
  mut spawn: EventWriter<SpawnParticle>,// <callout id="elements.event_spawn_particle" />
)
 {
  let mut collected = Vec::new();// <callout id="elements.collected" />
  for collision in collisions.read() {
    collected.push(collision.entity_b);
  }

  let Ok((mut player, player_pos)) = player.single_mut() else {
    return;
  };

  for miner in collected.iter() {// <callout id="elements.despawn" />
    if commands.get_entity(*miner).is_ok() {
      commands.entity(*miner).despawn();
    }
    T::effect(&mut player);// <callout id="elements.generic_effect" />
  }

  if !collected.is_empty() {
    // Spawn burst of particles
    particle_burst(
      player_pos.translation.truncate(), 
      BurstColor::from(COLOR).into(),
      &mut spawn,
      2.0,
    );
  }
}
//END: CollectGameElement

//START: MinerBeacon
fn miner_beacon(
  mut rng: ResMut<RandomNumberGenerator>,
  miners: Query<&Transform, With<Miner>>,
  mut spawn: EventWriter<SpawnParticle>, 
) {
  for miner in miners.iter() {
    if rng.range(1 ..= 100) == 100 {
      particle_burst(
        miner.translation.truncate(),
        LinearRgba::new(1.0, 1.0, 0.0, 1.0),
        &mut spawn, 
        10.0)
    }
  }
}
//END: MinerBeacon

//START: ScoreDisplay
fn score_display(
  player: Query<&Player>,
  mut egui_context: egui::EguiContexts,
) {
  let Ok(player) = player.single() else {
    return;
  };
  egui::egui::Window::new("Score").show(
    egui_context.ctx_mut(),
    |ui| {
      ui.label(format!("Miners Saved: {}", player.miners_saved));
      ui.label(format!("Shields: {}", player.shields));
      ui.label(format!("Fuel: {}", player.fuel));
    });
}
//END: ScoreDisplay

//START: ParticleBurst
fn particle_burst(
  center: Vec2, 
  color: LinearRgba,
  spawn: &mut EventWriter<SpawnParticle>,
  velocity: f32,
) {
  for angle in 0 .. 360 {
    let angle = (angle as f32).to_radians();
    let velocity = Vec3::new(angle.cos() * velocity, 
      angle.sin() * velocity, 0.0);
    spawn.write(SpawnParticle {
      position: center,
      color,
      velocity,
    });
  }
}
//END: ParticleBurst

//START: CollectableComponents
#[derive(Component)]
struct Miner;

#[derive(Component)]
struct Battery;

#[derive(Component)]
struct Fuel;
//END: CollectableComponents