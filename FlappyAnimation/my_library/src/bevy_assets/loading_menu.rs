//START: use
use bevy::{prelude::*, asset::LoadedUntypedAsset};
use bevy::state::state::FreelyMutableState;
use bevy_egui::EguiContexts;
use crate::{AssetStore, egui::egui::Window, MenuResource, LoadedAssets, AssetManager};
use crate::bevy_assets::setup_asset_store;
//END: use

//START: setup
#[derive(Resource)]
pub(crate) struct AssetsToLoad(Vec<Handle<LoadedUntypedAsset>>);

pub(crate) fn setup(
    assets: Option<Res<AssetStore>>,
    asset_manager: Option<Res<AssetManager>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let assets = match assets {
        Some(assets) => assets.into_inner(),
        None => {
            &setup_asset_store(asset_manager.as_ref().unwrap(), &mut commands, &asset_server)
        }
    };
    let assets_to_load: Vec<Handle<LoadedUntypedAsset>> = assets
        .asset_index.values().cloned().collect();
    commands.insert_resource(AssetsToLoad(assets_to_load));
}
//END: setup

//START: run
pub(crate) fn run<T>(
    asset_server: Res<AssetServer>,
    mut to_load: ResMut<AssetsToLoad>,
    mut state: ResMut<NextState<T>>,
    mut egui_context: EguiContexts,
    menu_info: Res<MenuResource<T>>,
    // START_HIGHLIGHT
    mut store: ResMut<AssetStore>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
    loaded_assets: Res<LoadedAssets>,
    // END_HIGHLIGHT
) where T: States+FromWorld+FreelyMutableState,
{
    to_load.0.retain(|handle| {
        match asset_server.get_load_state(handle.id()) {
            Some(bevy::asset::LoadState::Loaded) => false,
            _ => true,
        }
    });
    //START: finished_loading
    if to_load.0.is_empty() {
        // START_HIGHLIGHT
        load_atlases(&mut store, &mut texture_atlases, &loaded_assets);
        // END_HIGHLIGHT
        state.set(menu_info.menu_state.clone());
    }
    //END: finished_loading
    Window::new("Loading, Please Wait").show(
        egui_context.ctx_mut(), |ui| {
            ui.label(
                format!("{} assets remaining", to_load.0.len())
            )
      });
}
//END: run

//START: load_atlases
fn load_atlases(
    store: &mut AssetStore,
    texture_atlases: &mut Assets<TextureAtlasLayout>,
    loaded_assets: &LoadedAssets,
) {
    for new_atlas in store.atlases_to_build.iter() {
        let atlas = TextureAtlasLayout::from_grid(
            new_atlas.tile_size.as_uvec2(),
            new_atlas.sprites_x as u32,
            new_atlas.sprites_y as u32,
            None, None);
        let atlas_handle = texture_atlases.add(atlas);
        let img = store.get_handle(&new_atlas.texture_tag, loaded_assets).unwrap();
        store
            .atlases
            .insert(new_atlas.tag.clone(), (img, atlas_handle));
    }
}
//END: load_atlases

//START: end
pub(crate) fn exit(
    mut commands: Commands,
) {
    commands.remove_resource::<AssetsToLoad>();
}
//END: end