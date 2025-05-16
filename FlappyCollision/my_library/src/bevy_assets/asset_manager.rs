use crate::{AssetStore, FutureAtlas};
use bevy::prelude::*;

//START: AssetType
#[derive(Clone)]
pub enum AssetType {
  Image,
  Sound,
  //START_HIGHLIGHT
  SpriteSheet{tile_size: Vec2, sprites_x: usize, sprites_y: usize},
  //END_HIGHLIGHT
}
//END: AssetType

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

  //START: AddSpriteSheet
  pub fn add_sprite_sheet<S: ToString>(
    mut self,
    tag: S,
    filename: S,
    sprite_width: f32,
    sprite_height: f32,
    sprites_x: usize,
    sprites_y: usize,
  ) -> anyhow::Result<Self> {
    let filename = filename.to_string();
    AssetManager::asset_exists(&filename)?;
    self
        .asset_list
        .push((tag.to_string(), filename, AssetType::SpriteSheet{
          tile_size: Vec2::new(
            sprite_width,
            sprite_height),
          sprites_x,
          sprites_y,
        }));
    Ok(self)
  }
  //END: AddSpriteSheet
}

//START: uncomment
impl Plugin for AssetManager {
  fn build(&self, app: &mut bevy::prelude::App) {
    app.insert_resource(self.clone());
  }
}
//END: uncomment

pub(crate) fn setup_asset_store(
  asset_resource: &AssetManager,
  commands: &mut Commands,
  asset_server: &AssetServer,
)-> AssetStore {//END: AssetSetupSig
  let mut assets = AssetStore {//<callout id="asset_mgr_init_store" />
    asset_index: bevy::platform::collections::HashMap::new(),
    atlases_to_build: Vec::new(),
    atlases: bevy::platform::collections::HashMap::new(),
  };
  //START: spawn_sound
  asset_resource.asset_list.iter().for_each(
    |(tag, filename, asset_type)| {
      //START: AssetSetup
      match asset_type {
        AssetType::SpriteSheet { tile_size, sprites_x, sprites_y } => {
          // Sprite Sheets require that we load the image first, and defer
          // sheet creation to the loading menu - after the image has loaded
          let image_handle = asset_server.load_untyped(filename);//<callout id="animation.load_image" />
          let base_tag = format!("{tag}_base");//<callout id="animation.tag_base_image" />
          assets
              .asset_index
              .insert(base_tag.clone(), image_handle);//<callout id="animation.insert_image" />

          // Now that its loaded, we store the future atlas in the asset store
          assets.atlases_to_build.push(FutureAtlas {//<callout id="animation.future_atlas" />
            tag: tag.clone(),
            texture_tag: base_tag,
            tile_size: *tile_size,
            sprites_x: *sprites_x,
            sprites_y: *sprites_y,
          });
        }
        //END: AssetSetup
        _ => {
          // Most asset types don't require a separate loader
          //END: spawn_sound
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