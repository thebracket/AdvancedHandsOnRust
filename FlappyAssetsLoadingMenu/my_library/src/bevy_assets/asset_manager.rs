use crate::AssetStore;
use bevy::prelude::*;

//START: asset_type_enum
#[derive(Clone)]
pub enum AssetType {
  Image,
  //START_HIGHLIGHT
  Sound,
  //END_HIGHLIGHT
}
//END: asset_type_enum

//START: asset_manager_struct
#[derive(Resource, Clone)]// <callout id="asset_manager_resource" />
pub struct AssetManager {
  asset_list: Vec<(String, String, AssetType)>, // <callout id="asset_list_tuple" />
}
//END: asset_manager_struct

//START: asset_manager_new
impl AssetManager {
  pub fn new() -> Self {
    Self {
      //START_HIGHLIGHT
      asset_list: vec![
        ("main_menu".to_string(), "main_menu.png".to_string(), 
          AssetType::Image),
        ("game_over".to_string(), "game_over.png".to_string(), 
          AssetType::Image),
      ],
      //END_HIGHLIGHT
    }
  }
  //END: asset_manager_new

  //START: check_exists
  fn asset_exists(filename: &str) -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
      let current_directory = std::env::current_dir()?;
      let assets = current_directory.join("assets");
      let new_image = assets.join(filename);
      if !new_image.exists() {
        return Err(anyhow::Error::msg(format!(
          "{} not found in assets directory",
          &filename
        )));
      }
    }
    Ok(())
  }
  //END: check_exists

  //START: load_image
  pub fn add_image<S: ToString>(
    mut self,
    tag: S,
    filename: S,
  ) -> anyhow::Result<Self> {
    let filename = filename.to_string();
    //START_HIGHLIGHT
    AssetManager::asset_exists(&filename)?;
    //END_HIGHLIGHT
    self
      .asset_list
      .push((tag.to_string(), filename, AssetType::Image));
    Ok(self)
  }
  //END: load_image

  //START: load_sound
  pub fn add_sound<S: ToString>(
    mut self,
    tag: S,
    filename: S,
  ) -> anyhow::Result<Self> {
    let filename = filename.to_string();
    AssetManager::asset_exists(&filename)?;
    self
      .asset_list
      .push((tag.to_string(), filename, AssetType::Sound));
    Ok(self)
  }
  //END: load_sound
}

//START: uncomment
impl Plugin for AssetManager {
  fn build(&self, app: &mut bevy::prelude::App) {
    app.insert_resource(self.clone());
    //START_HIGHLIGHT
    //app.add_systems(Startup, setup);
    //END_HIGHLIGHT
  }
}
//END: uncomment

//START: asset_manager_setup
pub(crate) fn setup_asset_store(
  asset_resource: &AssetManager,//<callout id="asset_mgr_load_self" />
  commands: &mut Commands,
  asset_server: &AssetServer,
) -> AssetStore {
  let mut assets = AssetStore {//<callout id="asset_mgr_init_store" />
    asset_index: bevy::platform::collections::HashMap::new(),
  };
  asset_resource.asset_list.iter().for_each(
    |(tag, filename, asset_type)| {
      match asset_type {
        _ => {
          // Most asset types don't require a separate loader
          assets
            .asset_index
            .insert(tag.clone(), asset_server.load_untyped(filename));
        }
      }
    },
  );
  commands.remove_resource::<AssetManager>();//<callout id="asset_mgr_remove_self" />
  commands.insert_resource(assets.clone());//<callout id="asset_mgr_add_store" />
  assets
}
//END: asset_manager_setup
