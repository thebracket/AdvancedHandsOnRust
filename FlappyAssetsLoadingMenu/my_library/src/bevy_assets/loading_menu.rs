//START: use
use bevy::{prelude::*, asset::LoadedUntypedAsset};
use bevy::state::state::FreelyMutableState;
use bevy_egui::EguiContexts;
use crate::{AssetStore, egui::egui::Window, MenuResource, AssetManager};
use crate::bevy_assets::setup_asset_store;
//END: use

//START: setup
#[derive(Resource)]
pub(crate) struct AssetsToLoad(Vec<Handle<LoadedUntypedAsset>>);//<callout id="loading_menu.resource" />

pub(crate) fn setup(
    assets: Option<Res<AssetStore>>,
    asset_manager: Option<Res<AssetManager>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let assets = match assets {
        Some(assets) => assets.into_inner(),
        None => {
            &setup_asset_store(asset_manager.as_ref().unwrap(), 
            &mut commands, &asset_server)
        }
    };
    let assets_to_load: Vec<Handle<LoadedUntypedAsset>> = assets
        .asset_index.values().cloned().collect();//<callout id="loading_menu.handles_to_load" />
    commands.insert_resource(AssetsToLoad(assets_to_load));//<callout id="loading_menu.assets_to_load" />
}
//END: setup

//START: run
pub(crate) fn run<T>(
    asset_server: Res<AssetServer>,
    mut to_load: ResMut<AssetsToLoad>,
    mut state: ResMut<NextState<T>>,
    mut egui_context: EguiContexts,
    menu_info: Res<MenuResource<T>>,
) where T: States+FromWorld+FreelyMutableState,
{
    to_load.0.retain(|handle| {//<callout id="loading_menu.retain" />
        match asset_server.get_load_state(handle.id()) {//<callout id="loading_menu.get_load_state" />
            Some(bevy::asset::LoadState::Loaded) => false,//<callout id="loading_menu.loaded" />
            _ => true,//<callout id="loading_menu.not_loaded" />
        }
    });
    if to_load.0.is_empty() {//<callout id="loading_menu.proceed" />
        state.set(menu_info.menu_state.clone());
    }
    Window::new("Loading, Please Wait").show(//<callout id="loading_menu.inform" />
        egui_context.ctx_mut(), |ui| {
            ui.label(
                format!("{} assets remaining", to_load.0.len())
            )
      });
}
//END: run

//START: end
pub(crate) fn exit(
    mut commands: Commands,
) {
    commands.remove_resource::<AssetsToLoad>();
}
//END: end