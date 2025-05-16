use bevy::prelude::*;
use bevy_egui::{egui, EguiPlugin, EguiContexts};
use my_library::*;

//START: GamePhase
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default, States)]
enum GamePhase {
  //START_HIGHLIGHT
  #[default]
  MainMenu,
  //END_HIGHLIGHT
  Start,
  Player,
  Cpu,
  End,
  //START_HIGHLIGHT
  GameOver,
  //END_HIGHLIGHT
}
//END: GamePhase

//START: GameElement
#[derive(Component)]
pub struct GameElement;
//END: GameElement

//START: main
fn main() {
  let mut app = App::new();
   
  add_phase!(app, GamePhase, GamePhase::Start,//<callout id="macro_pig.start" />
    start => [ setup ], run => [ start_game ], exit => [ ]
  );

  add_phase!(app, GamePhase, GamePhase::Player,//<callout id="macro_pig.player" />
    start => [ ], run => [ player, check_game_over, display_score ]
      ,exit => [ ]
  );

  add_phase!(app, GamePhase, GamePhase::Cpu,//<callout id="macro_pig.cpu" />
    start => [ ], run => [ cpu, check_game_over, display_score ]
      , exit => [ ]
  );

  add_phase!(app, GamePhase, GamePhase::End,//<callout id="macro_pig.end" />
    start => [ ], run => [ end_game ], exit => [ cleanup::<GameElement> ]
  );

  add_phase!(app, GamePhase, GamePhase::GameOver,//<callout id="macro_pig.game_over" />
    start => [ ], run => [ display_final_score ], exit => [ ]
  );

  app.add_plugins(DefaultPlugins.set(
    WindowPlugin {
      primary_window: Some(Window {
      title: "Pig".to_string(),
      resolution: bevy::window::WindowResolution::new(1024.0, 768.0),
      ..default()
    }),
    ..default()
  }))
  .add_plugins(GameStatePlugin::new(GamePhase::MainMenu, GamePhase::Start, 
    GamePhase::GameOver))
  .add_plugins(EguiPlugin{ enable_multipass_for_primary_context: false })
  .add_plugins(RandomPlugin)
  .run();
}
//END: main

#[derive(Resource)]
struct GameAssets {
  image: Handle<Image>,
  layout: Handle<TextureAtlasLayout>,
}

#[derive(Clone, Copy, Resource)]
struct Scores {
  player: usize,
  cpu: usize,
}

//START: FinalScoreStruct
#[derive(Resource)]
struct FinalScore(Scores);
//END: FinalScoreStruct

#[derive(Component)]
struct HandDie;

#[derive(Resource)]
struct HandTimer(Timer);

//START: game_element1
fn setup(
  asset_server: Res<AssetServer>,
  mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
  mut commands: Commands,
) {
  //START_HIGHLIGHT
  commands.spawn(Camera2d::default()).insert(GameElement);
  //END_HIGHLIGHT
  //END: game_element1
  let texture = asset_server.load("dice.png");
  let layout = TextureAtlasLayout::from_grid(UVec2::splat(52), 6, 1, None, None);
  let texture_atlas_layout = texture_atlas_layouts.add(layout);
  commands.insert_resource(GameAssets { image: texture, layout: texture_atlas_layout });
  commands.insert_resource(Scores { cpu: 0, player: 0 });
  commands.insert_resource(HandTimer(Timer::from_seconds(0.5, TimerMode::Repeating)));
}

fn display_score(
  scores: Res<Scores>,
  mut egui_context: EguiContexts,
) {
  egui::Window::new("Total Scores").show(egui_context.ctx_mut(), |ui| {
    ui.label(&format!("Player: {}", scores.player));
    ui.label(&format!("CPU: {}", scores.cpu));
  });
}

fn clear_die(
  hand_query: &Query<(Entity, &Sprite), With<HandDie>>,
  commands: &mut Commands,
) {
  hand_query
    .iter()
    .for_each(|(entity, _)| commands.entity(entity).despawn());
}

//START: game_element2
fn spawn_die(
  hand_query: &Query<(Entity, &Sprite), With<HandDie>>,
  commands: &mut Commands,
  assets: &GameAssets,
  new_roll: usize,
  color: Color,
) {
  let rolled_die = hand_query.iter().count() as f32 * 52.0;
  let mut sprite = Sprite::from_atlas_image(
    assets.image.clone(),
    TextureAtlas {
      layout: assets.layout.clone(),
      index: new_roll - 1,
    }
  );
  sprite.color = color;

  commands.spawn((
    sprite,
    Transform::from_xyz(rolled_die - 400.0, 60.0, 1.0),
    HandDie,
    //START_HIGHLIGHT
    GameElement
    //END_HIGHLIGHT
  ));
  //END: game_element2
}

fn player(
  hand_query: Query<(Entity, &Sprite), With<HandDie>>,
  mut commands: Commands,
  mut rng: ResMut<RandomNumberGenerator>,
  assets: Res<GameAssets>,
  mut scores: ResMut<Scores>,
  mut state: ResMut<NextState<GamePhase>>,
  mut egui_context: EguiContexts,
) {
  egui::Window::new("Play Options").show(egui_context.ctx_mut(), |ui| {
    let hand_score: usize =
      hand_query.iter().map(|(_, ts)| ts.texture_atlas.as_ref().unwrap().index + 1).sum();
    ui.label(&format!("Score for this hand: {}", hand_score));

    if ui.button("Roll Dice").clicked() {
      let new_roll = rng.range(1..=6);
      if new_roll == 1 {
        // End turn!
        clear_die(&hand_query, &mut commands);
        state.set(GamePhase::Cpu);
      } else {
        spawn_die(
          &hand_query,
          &mut commands,
          &assets,
          new_roll,
          Color::WHITE,
        );
      }
    }
    if ui.button("Pass - Keep Hand Score").clicked() {
      let hand_total: usize =
        hand_query.iter().map(|(_, ts)| ts.texture_atlas.as_ref().unwrap().index + 1).sum();
      scores.player += hand_total;
      clear_die(&hand_query, &mut commands);
      state.set(GamePhase::Cpu);
    }
  });
}

fn cpu(
  hand_query: Query<(Entity, &Sprite), With<HandDie>>,
  mut state: ResMut<NextState<GamePhase>>,
  mut scores: ResMut<Scores>,
  mut rng: ResMut<RandomNumberGenerator>,
  mut commands: Commands,
  assets: Res<GameAssets>,
  mut timer: ResMut<HandTimer>,
  time: Res<Time>,
) {
  timer.0.tick(time.delta());
  if timer.0.just_finished() {
    let hand_total: usize =
      hand_query.iter().map(|(_, ts)| ts.texture_atlas.as_ref().unwrap().index + 1).sum();
    if hand_total < 20 && scores.cpu + hand_total < 100 {
      let new_roll = rng.range(1..=6);
      if new_roll == 1 {
        clear_die(&hand_query, &mut commands);
        state.set(GamePhase::Player);
      } else {
        spawn_die(
          &hand_query,
          &mut commands,
          &assets,
          new_roll,
          Color::Srgba(Srgba::new(0.0, 0.0, 1.0, 1.0)),
        );
      }
    } else {
      scores.cpu += hand_total;
      state.set(GamePhase::Player);
      hand_query
        .iter()
        .for_each(|(entity, _)| commands.entity(entity).despawn());
    }
  }
}

//START: IsGameOver
fn check_game_over(
  mut state: ResMut<NextState<GamePhase>>,
  scores: Res<Scores>,
) {
  if scores.cpu >= 100 || scores.player >= 100 {
    state.set(GamePhase::End);
  }
}
//END: IsGameOver

//START: StartGame
fn start_game(mut state: ResMut<NextState<GamePhase>>) {
  state.set(GamePhase::Player);
}
//END: StartGame

//START: EndGame
fn end_game(
  mut state: ResMut<NextState<GamePhase>>, 
  scores: Res<Scores>,
  mut commands: Commands,
) {
  commands.insert_resource(FinalScore(*scores));
  state.set(GamePhase::GameOver);
}
//END: EndGame

//START: FinalScore
fn display_final_score(
  scores: Res<FinalScore>,
  mut egui_context: EguiContexts,
) {
  egui::Window::new("Total Scores").show(egui_context.ctx_mut(), |ui| {
    ui.label(&format!("Player: {}", scores.0.player));
    ui.label(&format!("CPU: {}", scores.0.cpu));
    if scores.0.player < scores.0.cpu {
      ui.label("CPU is the winner!");
    } else {
      ui.label("Player is the winnner!");
    }
  });
}
//END: FinalScore