use bevy::prelude::*;
use bevy::state::state::FreelyMutableState;

mod game_menus;

pub struct GameStatePlugin<T> {
  menu_state: T,
  game_start_state: T,
  game_end_state: T,
}

impl<T> GameStatePlugin<T>
where
  T: States+FromWorld+FreelyMutableState,
{
  #[allow(clippy::new_without_default)]
  pub fn new(menu_state: T, game_start_state: T, game_end_state: T) -> Self {
      Self { menu_state, game_start_state, game_end_state }
  }
}

//START: build
impl<T> Plugin for GameStatePlugin<T>
where
  T: States+Copy+FromWorld+FreelyMutableState,
{
  fn build(&self, app: &mut App) {
    app.init_state::<T>();//<callout id="flappy_menus.start_state" />
    app.add_systems(Startup, setup_menus);//<callout id="flappy_menus.load_menus" />
    let start = MenuResource {
      menu_state: self.menu_state,
      game_start_state: self.game_start_state,
      game_end_state: self.game_end_state,
    };
    app.insert_resource(start);

    app.add_systems(OnEnter(self.menu_state), game_menus::setup::<T>);
    app.add_systems(Update, game_menus::run::<T>.run_if(in_state(self.menu_state)));
    app.add_systems(OnExit(self.menu_state), cleanup::<game_menus::MenuElement>);

    app.add_systems(OnEnter(self.game_end_state), game_menus::setup::<T>);
    app.add_systems(Update, game_menus::run::<T>.run_if(in_state(self.game_end_state)));
    app.add_systems(OnExit(self.game_end_state), cleanup::<game_menus::MenuElement>);
  }
}
//END: build

//START: load_assets
#[derive(Resource)]
pub(crate) struct MenuAssets {
  pub(crate) main_menu: Handle<Image>,//<callout id="flappy_menus.pub_crate" />
  pub(crate) game_over: Handle<Image>,
}

fn setup_menus(mut commands: Commands, asset_server: Res<AssetServer>) {
  let assets = MenuAssets {
    main_menu: asset_server.load("main_menu.png"),
    game_over: asset_server.load("game_over.png"),
  };
  commands.insert_resource(assets);
}
//END: load_assets

//START: menu_resource
#[derive(Resource)]
pub(crate) struct MenuResource<T> {
  pub(crate) menu_state: T,
  pub(crate) game_start_state: T,
  pub(crate) game_end_state: T,
}
//END: menu_resource

pub fn cleanup<T>(query: Query<Entity, With<T>>, mut commands: Commands)
where
  T: Component,
{
  query.iter().for_each(|entity| commands.entity(entity).despawn())
}

//START: macro_set
#[macro_export]
macro_rules! add_phase {
  (
    $app:expr, $type:ty, $phase:expr,
    start => [ $($start:expr),* ],
    run => [ $($run:expr),* ],
    exit => [ $($exit:expr),* ]
  ) => {
    $($app.add_systems(
      bevy::prelude::OnEnter::<$type>($phase),
      $start
    ))*;
    $($app.add_systems(
      bevy::prelude::Update, $run.run_if(in_state($phase))
    );)*
    $($app.add_systems(
      bevy::prelude::OnExit::<$type>($phase),
      $exit
    );)*

  };
}
//END: macro_set