use bevy::{
  diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
  prelude::*,
  platform::collections::{HashMap, HashSet},
};
use my_library_flappy_collision::{egui::egui::Color32, *};

const QUAD_TREE_DEPTH: usize = 6;

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
struct CollisionTime {
  time: u128,
  checks: u32,
}

#[derive(Component)]
struct Ball;

#[derive(Component)]
struct AxisAlignedBoundingBox {
  half_size: Vec2,
}


impl AxisAlignedBoundingBox {
  pub fn new(width: f32, height: f32) -> Self {
    Self {
      half_size: Vec2::new(width / 2.0, height / 2.0),
    }
  }

  fn as_rect(&self, translate: Vec2) -> Rect2D {
    Rect2D::new(
      Vec2::new(translate.x - self.half_size.x, translate.y - self.half_size.y),
      Vec2::new(translate.x + self.half_size.x, translate.y + self.half_size.y),
    )
  }
}

#[derive(Debug, Clone, Copy)]
struct Rect2D {
  min: Vec2,
  max: Vec2,
}

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
//START: quadrants
  fn quadrants(&self) -> Vec<Self> {
    let center = (self.min + self.max) / 2.0;
    vec![
      Self::new(self.min, center), // Top-left
      Self::new(
        Vec2::new(center.x, self.min.y),
        Vec2::new(self.max.x, center.y),
      ), // Top-Right
      Self::new(
        Vec2::new(self.min.x, center.x),
        Vec2::new(center.x, self.max.y),
      ), // Bottom-left
      Self::new(center, self.max), // Bottom-right
    ]
  }
}
//END: quadrants

//START: quadtree
#[derive(Debug, Resource)]
pub struct StaticQuadTree {
  nodes: Vec<StaticQuadTreeNode>,
}
//END: quadtree

//START: quadtreenode
#[derive(Debug)]
pub struct StaticQuadTreeNode {
  bounds: Rect2D,
  children: Option<[usize; 4]>,
}
//END: quadtreenode

//START: staticquadtreenew
impl StaticQuadTree {
  fn new(screen_size: Vec2, max_depth: usize) -> Self {
    // Make the container
    let mut nodes = Vec::new();

    // Create the top-level, always the whole screen
    let half = screen_size / 2.0;
    let top = StaticQuadTreeNode {
      bounds: Rect2D::new(
        Vec2::new(0.0 - half.x, 0.0 - half.y),
        Vec2::new(half.x, half.y),
      ),
      children: None,
    };
    nodes.push(top);// <callout id="quadtree.push_top" />
    Self::subdivide(&mut nodes, 0, 1, max_depth);// <callout id="quadtree.call_subdivide" />
    Self { nodes }
  }
  //END: staticquadtreenew

//START: smallestnode
  fn smallest_node(&self, target: &Rect2D) -> usize {
    let mut current_index = 0;

    #[allow(clippy::while_let_loop)]// <callout id="quadtree.clippy_allow" />
    loop {// <callout id="quadtree.small_loop" />
      if let Some(children) = self.nodes[current_index].children {// <callout id="quadtree.small_iflet" />
        let matches: Vec<usize> = children
          .iter()
          .filter_map(|child| {
            if self.nodes[*child].bounds.intersect(target) {
              Some(*child)
            } else {
              None
            }
          })
          .collect();// <callout id="quadtree.small_collect" />

        if matches.len() == 1 {// <callout id="quadtree.small_len" />
          current_index = matches[0];
        } else {
          break;
        }
      } else {
        break;
      }
    }

    current_index
  }
//END: smallestnode

//START: staticquadtreesubdivide
  fn subdivide(
    nodes: &mut Vec<StaticQuadTreeNode>,// <callout id="quadtree.subdivide_nodes" />
    index: usize,// <callout id="quadtree.subdivide_index" />
    depth: usize,// <callout id="quadtree.subdivide_depth" />
    max_depth: usize,
  ) {
    let mut children = nodes[index].bounds.quadrants();// <callout id="quadtree.subdivide_quadrants" />
    let child_index = [
      nodes.len(),
      nodes.len() + 1,
      nodes.len() + 2,
      nodes.len() + 3,
    ];
    nodes[index].children = Some(child_index);// <callout id="quadtree.subdivide_child_indices" />
    children.drain(0..4).for_each(|quad| {// <callout id="quadtree.subdivide_drain" />
      nodes.push(StaticQuadTreeNode {
        bounds: quad,
        children: None,
      })
    });

    if depth < max_depth {// <callout id="quadtree.depth_test" />
      for index in child_index {
        Self::subdivide(nodes, index, depth + 1, max_depth);
      }
    }
  }
  //END: staticquadtreesubdivide

  //START: intersecting_nodes
  fn intersecting_nodes(&self, target: &Rect2D) -> HashSet<usize> {
    let mut result = HashSet::new();
    self.intersect(0, &mut result, target);
    result
  }

  fn intersect(
    &self,
    index: usize,
    result: &mut HashSet<usize>,
    target: &Rect2D,
  ) {
    if self.nodes[index].bounds.intersect(target) {
      result.insert(index);
      if let Some(children) = &self.nodes[index].children {
        for child in children {
          self.intersect(*child, result, target);
        }
      }
    }
  }
  //END: intersecting_nodes
}


fn main() -> anyhow::Result<()> {
  let mut app = App::new();
  add_phase!(app, GamePhase, GamePhase::Bouncing,
    start => [ setup ],
    run => [ warp_at_edge, collisions, show_performance,
      continual_parallax, physics_clock, sum_impulses, apply_velocity ],
    exit => [ cleanup::<BouncyElement> ]
  );

  app.add_plugins(DefaultPlugins.set(WindowPlugin {
    primary_window: Some(Window {
      title: "QuadTree Box Collision".to_string(),
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
      loaded_assets,
      BouncyElement,
      Velocity::new(velocity.x, velocity.y, velocity.z),
      AxisAlignedBoundingBox::new(8.0, 8.0),
      Ball
    );
  }
}

//START: setup
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
  //START_HIGHLIGHT
  commands
    .insert_resource(StaticQuadTree::new(Vec2::new(1024.0, 768.0), 
      QUAD_TREE_DEPTH));
  //END_HIGHLIGHT
  spawn_bouncies(1, &mut commands, &mut rng, &assets, &loaded_assets);
}
//END: setup

fn warp_at_edge(
  mut query: Query<&mut Transform>,
  //mut force: EventWriter<Impulse>,
  mut rng: ResMut<RandomNumberGenerator>,
) {
  for mut transform in query.iter_mut() {
    if transform.translation.y < -380.0
      || transform.translation.y > 380.0
      || transform.translation.x < -508.0
      || transform.translation.x > 508.0
    {
      transform.translation =
        Vec3::new(rng.range(-512.0..512.0), rng.range(-384.0..384.0), 0.0);
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
) {
  let n_balls = query.iter().count();
  let fps = diagnostics
    .get(&FrameTimeDiagnosticsPlugin::FPS)
    .and_then(|fps| fps.average())
    .unwrap();
  egui::egui::Window::new("Performance").show(
    egui_context.ctx_mut(),
    |ui| {
      let fps_text = format!("FPS: {fps:.1}");
      let color = match fps as u32 {
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

//START: collisions_sig
fn collisions(
  mut collision_time: ResMut<CollisionTime>,
  query: Query<(Entity, &Transform, &AxisAlignedBoundingBox)>, // <callout id="co.quadtree.query" />
  mut impulse: EventWriter<Impulse>,
  quad_tree: Res<StaticQuadTree>,// <callout id="co.quadtree.res" />
) {
//END: collisions_sig
//START: collisions_build_tree
  // Start the clock
  let now = std::time::Instant::now();
  
  let mut spatial_index: HashMap<usize, Vec<(Entity, Rect2D)>> =
    HashMap::new();// <callout id="co.quadtree.spatial_index" />

  let tree_positions: Vec<(Entity, usize, Rect2D)> = query// <callout id="co.quadtree.use_query" />
    .iter()
    .map(|(entity, transform, bbox)| {
      let bbox = bbox.as_rect(transform.translation.truncate());// <callout id="co.quadtree.bbox_trans" />
      let node = quad_tree.smallest_node(&bbox);// <callout id="co.quadtree.find_node" />
      for in_node in quad_tree.intersecting_nodes(&bbox) {// <callout id="co.quadtree.intsersecting_nodes" />
        if let Some(contents) = spatial_index.get_mut(&in_node) {// <callout id="co.quadtree.if_exists" />
          contents.push((entity, bbox));
        } else {
          spatial_index.insert(in_node, vec![(entity, bbox)]);
        }
      }

      (entity, node, bbox)
    })
    .collect();// <callout id="co.quadtree.collect_tree_positions" />
//END: collisions_build_tree
//START: collision_collide
  let mut n = 0;

  for (entity, node, box_a) in tree_positions {
    if let Some(entities_here) = spatial_index.get(&node) {
      if let Some((entity_b, _)) = entities_here
        .iter()
        .filter(|(entity_b, _)| *entity_b != entity)
        .find(|(_, box_b)| {
          n += 1;
          box_a.intersect(box_b)
        })
      {
        // A Collision occurred
        let (_, ball_a, _) = query.get(entity).unwrap();
        let (_, ball_b, _) = query.get(*entity_b).unwrap();
        bounce_on_collision(entity, ball_a.translation, 
          ball_b.translation, &mut impulse);
      }
    }
  }

  // Store the time result
  collision_time.time = now.elapsed().as_millis();
  collision_time.checks = n;
}
//END: collision_collide