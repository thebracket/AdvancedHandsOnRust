mod asset_manager;

pub use asset_manager::AssetManager;
mod asset_store;
pub use asset_store::*;

//START: spawn_image_macro
#[macro_export]
macro_rules! spawn_image {
 ($assets:expr, $commands:expr, $index:expr, $x:expr, $y:expr, $z:expr, 
   $resource:expr, 
    $($component:expr),*) => 
{
    $commands.spawn((
        Sprite::from_image($assets.get_handle($index, $resource).unwrap()),
        Transform::from_xyz($x, $y, $z),
    ))
    $(
      .insert($component)
    )*
 };
}
//END: spawn_image_macro
