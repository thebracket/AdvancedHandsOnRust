use bevy::prelude::*;
use bevy::state::state::FreelyMutableState;
use bevy_egui::EguiPrimaryContextPass;

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
      Self { menu_state, game_start_state, game_end_state } //<callout id="generic_state.assign_playing" />
  }
}

//START: build
impl<T> Plugin for GameStatePlugin<T>
where
  T: States+Copy+FromWorld+FreelyMutableState+Default,
{
  //START: run_loader
  fn build(&self, app: &mut App) {
    app.init_state::<T>();    
    //START_HIGHLIGHT
    app.add_plugins(bevy_egui::EguiPlugin::default());
    //END_HIGHLIGHT
    let start = MenuResource {
      menu_state: self.menu_state,
      game_start_state: self.game_start_state,
      game_end_state: self.game_end_state,
    };
    app.insert_resource(start);

    app.add_systems(OnEnter(self.menu_state), game_menus::setup::<T>);
    app.add_systems(Update, game_menus::run::<T>
      .run_if(in_state(
      self.menu_state)));
    app.add_systems(OnExit(self.menu_state), 
      cleanup::<game_menus::MenuElement>);

    app.add_systems(OnEnter(self.game_end_state), game_menus::setup::<T>);
    app.add_systems(Update, game_menus::run::<T>
      .run_if(in_state(self.game_end_state)));
    app.add_systems(OnExit(self.game_end_state), 
      cleanup::<game_menus::MenuElement>);

    app.add_systems(OnEnter(T::default()), crate::bevy_assets::setup);
    app.add_systems(EguiPrimaryContextPass, crate::bevy_assets::run::<T>
      .run_if(in_state(T::default())));
    app.add_systems(OnExit(T::default()), crate::bevy_assets::exit);
  }
    //END: run_loader
  }
//END: build

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