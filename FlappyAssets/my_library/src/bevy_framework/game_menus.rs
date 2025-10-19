//START: use
use super::{MenuAssets, MenuResource};
use bevy::{app::AppExit, prelude::*};
use bevy::state::state::FreelyMutableState;
//END: use

//START: tag
#[derive(Component)]
pub(crate) struct MenuElement;
//END: tag

//START: setup
pub(crate) fn setup<T>(// <callout id="flappy_menu.menu_setup" />
  state: Res<State<T>>,
  mut commands: Commands,
  menu_resource: Res<MenuResource<T>>,
  asset_server: Res<AssetServer>,
) where
  T: States+FromWorld+FreelyMutableState,
{
  let menu_assets = MenuAssets {
    main_menu: asset_server.load("main_menu.png"),
    game_over: asset_server.load("game_over.png"),
  };
  let current_state = state.get();
  let menu_graphic = {
    if menu_resource.menu_state == *current_state {
      menu_assets.main_menu.clone()
    } else if menu_resource.game_end_state == *current_state {
      menu_assets.game_over.clone()
    } else {
      panic!("Unknown menu state")
    }
  };

  commands
    .spawn(Camera2d::default())
    .insert(MenuElement);
  commands
      .spawn((
          Sprite::from_image(menu_graphic.clone()),
            Transform::from_xyz(0.0, 0.0, 1.0),
            MenuElement,
          ));
}
//END: setup

//START: run
pub(crate) fn run<T>(
  keyboard: Res<ButtonInput<KeyCode>>,
  mut exit: EventWriter<AppExit>,
  current_state: Res<State<T>>,
  mut state: ResMut<NextState<T>>,
  menu_state: Res<MenuResource<T>>,
) where
  T: States+FromWorld+FreelyMutableState,
{
  let current_state = current_state.get().clone();
  if current_state == menu_state.menu_state {
      if keyboard.just_pressed(KeyCode::KeyP) {
        state.set(menu_state.game_start_state.clone());
      } else if keyboard.just_pressed(KeyCode::KeyQ) {
        exit.write(AppExit::Success);
      }
    }
    else if current_state == menu_state.game_end_state {
      if keyboard.just_pressed(KeyCode::KeyM) {
        state.set(menu_state.menu_state.clone());
      } else if keyboard.just_pressed(KeyCode::KeyQ) {
        exit.write(AppExit::Success);
      }
    }
}
//END: run
