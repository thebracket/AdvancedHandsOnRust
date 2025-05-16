//START: asset_store_use
use bevy::{asset::{Asset, LoadedUntypedAsset}, prelude::*, platform::collections::HashMap};
//END: asset_store_use

//START: asset_store_types
pub type LoadedAssets = Assets<LoadedUntypedAsset>;
pub type AssetResource<'w> = Res<'w, LoadedAssets>;
//END: asset_store_types

//START: asset_store_struct
#[derive(Resource)]//<callout id="asset_store_resource" />
pub struct AssetStore {
  pub(crate) asset_index: HashMap<String, Handle<LoadedUntypedAsset>>,//<callout id="asset_store_handle_untyped" />
}
//END: asset_store_struct

//START: asset_store_get
impl AssetStore {
  pub fn get_handle<T>(&self, index: &str, assets: &LoadedAssets) 
  -> Option<Handle<T>>//<callout id="asset_store_generic_types" />
  where T: Asset,//<callout id="asset_store_generic_type" />
  {
    if let Some(handle_untyped) = self.asset_index.get(index) {//<callout id="asset_store_if_let" />
      if let Some(handle) = assets.get(handle_untyped) {//<callout id="asset_store_get" />
        return Some(handle.handle.clone().typed::<T>());//<callout id="asset_store_deduce" />
      }
      None
    } else {
      None//<callout id="asset_store_none" />
    }
  }
}
//END: asset_store_get