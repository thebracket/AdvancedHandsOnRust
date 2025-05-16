mod asset_manager;
pub use asset_manager::*;
mod asset_store;
pub use asset_store::*;
mod loading_menu;
pub(crate) use loading_menu::*;

#[macro_export]
macro_rules! spawn_image {
 ($assets:expr, $commands:expr, $index:expr, $x:expr, $y:expr, $z:expr, 
   $loaded_assets:expr,
    $($component:expr),*) => 
{
    $commands.spawn((
        Sprite::from_image($assets.get_handle($index, $loaded_assets).unwrap()),
        Transform::from_xyz($x, $y, $z),
    ))
    $(
      .insert($component)
    )*
 };
}