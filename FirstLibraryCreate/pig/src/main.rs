//START: main
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin}; //<callout id="first_library_create.pig.egui" />
use my_library::RandomNumberGenerator; //<callout id="first_library_create.pig.use" />

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default, States)] //<callout id="first_library_create.pig.state" />
enum GamePhase {
  #[default]
  Player,
  Cpu,
}

fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_plugins(EguiPlugin{ enable_multipass_for_primary_context: false })
    .add_systems(Startup, setup) //<callout id="first_library_create.pig.call_setup" />
    .init_state::<GamePhase>() //<callout id="first_library_create.pig.setup_state" />
    .add_systems(Update, display_score) //<callout id="first_library_create.pig.call_score" />
    .add_systems(Update, player.run_if(
      in_state(GamePhase::Player))) //<callout id="first_library_create.pig.call_player_update" />
    .add_systems(Update, cpu.run_if(
      in_state(GamePhase::Cpu))) //<callout id="first_library_create.pig.call_cpu_update" />
    .run();
}
//END: main

//START: components
#[derive(Resource)]
struct GameAssets {
  image: Handle<Image>,
  layout: Handle<TextureAtlasLayout>,
}

#[derive(Clone, Copy, Resource)]
struct Scores {
  player: usize,//<callout id="first_library_create.pig.scores" />
  cpu: usize,
}

#[derive(Component)] //<callout id="first_library_create.pig.hand_die_tag" />
struct HandDie;

#[derive(Resource)]
struct Random(RandomNumberGenerator);//<callout id="first_library_create.pig.rng_wrap" />

#[derive(Resource)]
struct HandTimer(Timer);//<callout id="first_library_create.pig.timer_wrap" />
//END: components

//START: setup
fn setup(
  asset_server: Res<AssetServer>, //<callout id="first_library_create.pig.asset_server" />
  mut commands: Commands, //<callout id="first_library_create.pig.setup_commands" />
  mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>, // <callout id="first_library_create.pig.atlas_resource" />
) {
  commands.spawn(Camera2d::default()); //<callout id="first_library_create.pig.2d_camera" />

  let texture = asset_server.load("dice.png");//<callout id="first_library_create.pig.load_image" />
  let layout = TextureAtlasLayout::from_grid(UVec2::splat(52), 6, 1, None, None);//<callout id="first_library_create.pig.from_grid" />
  let texture_atlas_layout = texture_atlas_layouts.add(layout);//<callout id="first_library_create.pig.atlas_layout" />

  commands.insert_resource(
    GameAssets { image: texture, layout: texture_atlas_layout }
  ); //<callout id="first_library_create.pig.store_assets" />
  commands.insert_resource(Scores { cpu: 0, player: 0 }); //<callout id="first_library_create.pig.store_score" />
  commands.insert_resource(Random(RandomNumberGenerator::new())); //<callout id="first_library_create.pig.store_rng" />
  commands.insert_resource(HandTimer(Timer::from_seconds
    (0.5, TimerMode::Repeating))); //<callout id="first_library_create.pig.timer" />
}
//END: setup

//START: display_score
fn display_score(
  scores: Res<Scores>,
  mut egui_context: EguiContexts, //<callout id="first_library_create.pig.egui_ctx" />
) {
  egui::Window::new("Total Scores").show(egui_context.ctx_mut(), |ui| {//<callout id="first_library_create.pig.egui_window" />
    ui.label(&format!("Player: {}", scores.player)); //<callout id="first_library_create.pig.show_player_score" />
    ui.label(&format!("CPU: {}", scores.cpu));
  });
}
//END: display_score

//START: clear_die
fn clear_die(
  hand_query: &Query<(Entity, &Sprite), With<HandDie>>,
  commands: &mut Commands,
) {
  hand_query
    .iter()
    .for_each(|(entity, _)| commands.entity(entity).despawn());
}
//END: clear_die

//START: spawn_die
fn spawn_die(
  hand_query: &Query<(Entity, &Sprite), With<HandDie>>, //<callout id="first_library_create.pig.hand_query" />
  commands: &mut Commands,
  assets: &GameAssets,
  new_roll: usize,
  color: Color,
) {
  let rolled_die = hand_query.iter().count() as f32 * 52.0; //<callout id="first_library_create.pig.count_hand_die" />

  let mut sprite = Sprite::from_atlas_image( // <callout id="first_library_create.pig.spawn_die_bundle" />
    assets.image.clone(),// <callout id="first_library_create.pig.spawn_die_atlas_image" />
    TextureAtlas {
      layout: assets.layout.clone(),// <callout id="first_library_create.pig.spawn_atlas_layout" />
      index: new_roll - 1,// <callout id="first_library_create.pig.atlas_index" />
    }
  );
  sprite.color = color;// <callout id="first_library_create.pig.spawn_tint" />

  commands.spawn((// <callout id="first_library_create.pig.spawn" />
    sprite,// <callout id="first_library_create.pig.spawn_sprite" />
    Transform::from_xyz(rolled_die - 400.0, 60.0, 1.0),// <callout id="first_library_create.pig.die_bundle_pos" />
    HandDie// <callout id="first_library_create.pig.spawn_hand_die_tag" />
    )
  );
}
//END: spawn_die

//START: player
fn player(
  hand_query: Query<(Entity, &Sprite), With<HandDie>>,
  mut commands: Commands,
  mut rng: ResMut<Random>,
  assets: Res<GameAssets>,
  mut scores: ResMut<Scores>,
  mut state: ResMut<NextState<GamePhase>>,//<callout id="first_library_create.pig.next_state" />
  mut egui_context: EguiContexts,
) {
  egui::Window::new("Play Options").show(egui_context.ctx_mut(), |ui| {
    let hand_score: usize =
      hand_query.iter().map(|(_, ts)| ts.texture_atlas
      .as_ref().unwrap().index + 1).sum();//<callout id="first_library_create.pig.hand_score" />
    ui.label(&format!("Score for this hand: {hand_score}"));

    if ui.button("Roll Dice").clicked() {//<callout id="first_library_create.pig.roll_button" />
      let new_roll = rng.0.range(1..7);
      if new_roll == 1 {
        // End turn!
        clear_die(&hand_query, &mut commands);
        state.set(GamePhase::Cpu);//<callout id="first_library_create.pig.end_hand" />
      } else {
        spawn_die(
          &hand_query,
          &mut commands,
          &assets,
          new_roll as usize,
          Color::WHITE,
        );
      }
    }
    if ui.button("Pass - Keep Hand Score").clicked() {
      let hand_total: usize =
        hand_query.iter().map(|(_, ts)| ts.texture_atlas
          .as_ref().unwrap().index + 1).sum();
      scores.player += hand_total;
      clear_die(&hand_query, &mut commands);
      state.set(GamePhase::Cpu);
    }
  });
}
//END: player

//START: cpu
#[allow(clippy::too_many_arguments)]//<callout id="first_library_create.pig.allow_arguments" />
fn cpu(
  hand_query: Query<(Entity, &Sprite), With<HandDie>>,
  mut state: ResMut<NextState<GamePhase>>,
  mut scores: ResMut<Scores>,
  mut rng: ResMut<Random>,
  mut commands: Commands,
  assets: Res<GameAssets>,
  mut timer: ResMut<HandTimer>,
  time: Res<Time>,
) {
  timer.0.tick(time.delta());//<callout id="first_library_create.pig.timer_delta" />
  if timer.0.just_finished() {//<callout id="first_library_create.pig.timer_just_finished" />
    let hand_total: usize =
      hand_query.iter().map(|(_, ts)| ts.texture_atlas
      .as_ref().unwrap().index + 1).sum();
    if hand_total < 20 && scores.cpu + hand_total < 100 {//<callout id="first_library_create.pig.optimal_play" />
      let new_roll = rng.0.range(1..7);
      if new_roll == 1 {
        clear_die(&hand_query, &mut commands);
        state.set(GamePhase::Player);
      } else {
        spawn_die(
          &hand_query,
          &mut commands,
          &assets,
          new_roll as usize,
          Color::Srgba(Srgba::new(0.0, 0.0, 1.0, 1.0)),//<callout id="first_library_create.pig.tint" />
        );
      }
    } else {
      scores.cpu += hand_total;//<callout id="first_library_create.pig.cpu_pass" />
      state.set(GamePhase::Player);
      hand_query
        .iter()
        .for_each(|(entity, _)| commands.entity(entity).despawn());
    }
  }
}
//END: cpu
