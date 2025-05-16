
//START: use
use bevy::prelude::*;
use bevy::state::state::FreelyMutableState;
//<callout id="generic_state.bevy_prelude" />
//END: use

//START: GenericPluginDef
pub struct GameStatePlugin<T> {//<callout id="generic_state.spec_t" />
  menu_state: T,//<callout id="generic_state.impl_t" />
  game_start_state: T,
  game_end_state: T,
}
//END: GenericPluginDef

//START: GameStatePluginConst
impl <T> GameStatePlugin<T>//<callout id="generic_state.impl_plugin" />
{
  #[allow(clippy::new_without_default)]
  pub fn new(menu_state: T, game_start_state: T, game_end_state: T) -> Self 
  {
    Self { menu_state, game_start_state, game_end_state } //<callout id="generic_state.assign_playing" />
  }
}
//END: GameStatePluginConst

//START: GameStateBuild
impl<T: States+FromWorld+FreelyMutableState> Plugin for GameStatePlugin<T>
{
  fn build(&self, app: &mut App) {
    app.init_state::<T>();
  }
}
//END: GameStateBuild

//START: cleanup
pub fn cleanup<T>(//<callout id="generic_state.cleanup_t" />
  query: Query<Entity, With<T>>,//<callout id="generic_state.generic_query" />
  mut commands: Commands,
) 
where T: Component//<callout id="generic_state.t_comp" />
{
  query.iter().for_each(|entity| commands.entity(entity).despawn())
}
//END: cleanup