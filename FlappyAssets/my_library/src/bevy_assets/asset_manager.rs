use crate::AssetStore;
use bevy::prelude::*;

//START: asset_type_enum
#[derive(Clone)]
pub enum AssetType {
  Image,
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
      asset_list: Vec::new(),
    }
  }
  //END: asset_manager_new

  //START: asset_manager_add_image
  pub fn add_image<S: ToString>(//<callout id="add_image_to_string" />
    mut self,
    tag: S,
    filename: S,
  ) -> anyhow::Result<Self> {//<callout id="anyhow_result" />
    let filename = filename.to_string();//<callout id="convert_filename" />
    #[cfg(not(target_arch = "wasm32"))]//<callout id="not_wasm" />
    {
      let current_directory = std::env::current_dir()?;//<callout id="current_dir" />
      let assets = current_directory.join("assets");//<callout id="join_assets" />
      let new_image = assets.join(&filename);
      if !new_image.exists() {
        return Err(anyhow::Error::msg(format!(//<callout id="non_existent" />
          "{} not found in assets directory",
          &filename
        )));
      }
    }
    self//<callout id="add_asset" />
      .asset_list
      .push((tag.to_string(), filename, AssetType::Image));
    Ok(self)//<callout id="asset_manager_ok" />
  }
}
//END: asset_manager_add_image

//START: uncomment
impl Plugin for AssetManager {
  fn build(&self, app: &mut bevy::prelude::App) {
    app.insert_resource(self.clone());
    //START_HIGHLIGHT
    app.add_systems(Startup, setup);
    //END_HIGHLIGHT
  }
}
//END: uncomment

//START: asset_manager_setup
fn setup(
  asset_resource: Res<AssetManager>,//<callout id="asset_mgr_load_self" />
  mut commands: Commands,
  asset_server: Res<AssetServer>,
) {
  let mut assets = AssetStore {//<callout id="asset_mgr_init_store" />
    asset_index: bevy::platform::collections::HashMap::new(),
  };
  asset_resource.asset_list.iter().for_each(
    |(tag, filename, asset_type)| {
      match asset_type {//<callout id="asset_mgr_ignore_type" />
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
  commands.insert_resource(assets);//<callout id="asset_mgr_add_store" />
}
//END: asset_manager_setup
