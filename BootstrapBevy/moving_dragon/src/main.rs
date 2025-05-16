use bevy::prelude::*;

//START: main
fn main() {
  App::new()
    .add_plugins(DefaultPlugins)
    .add_systems(Startup, setup)
    //START_HIGHLIGHT
    .add_systems(Update, movement)
    //END_HIGHLIGHT
    .run();
}
//END:main
//START: component
#[derive(Component)]
struct Dragon;
//END: component

//START:setup
fn setup(//<callout id="bouncing_bevy.setup_fn" />
  mut commands: Commands, //<callout id="bouncing_bevy.setup_commands" />
  asset_server: Res<AssetServer>//<callout id="bouncing_bevy.asset_server" />
) {
  commands.spawn(Camera2d::default());//<callout id="bouncing_bevy.spawn_camera" />
  let dragon_image = asset_server.load("dragon.png");//<callout id="bouncing_bevy.load" />
  commands
    .spawn(Sprite::from_image(dragon_image))//<callout id="bouncing_bevy.spawn" />
    .insert(Dragon);//<callout id="bouncing_bevy.ball" />
}
//END:setup
//START:movement
fn movement(
  keyboard: Res<ButtonInput<KeyCode>>,//<callout id="bouncing_bevy.keyboard" />
  mut dragon_query: Query<&mut Transform, With<Dragon>>,//<callout id="bouncing_bevy.query" />
) {
  let delta = if keyboard.pressed(KeyCode::ArrowLeft) {//<callout id="bouncing_bevy.delta" />
    Vec2::new(-1.0, 0.0)
  } else if keyboard.pressed(KeyCode::ArrowRight) {
    Vec2::new(1.0, 0.0)
  } else if keyboard.pressed(KeyCode::ArrowDown) {
    Vec2::new(0.0, -1.0)
  } else if keyboard.pressed(KeyCode::ArrowUp) {
    Vec2::new(0.0, 1.0)
  } else {
    Vec2::ZERO
  };

  dragon_query.iter_mut().for_each(|mut transform| {//<callout id="bouncing_bevy.for_each_mut" />
    transform.translation += delta.extend(0.0);//<callout id="bouncing_bevy.transform" />
  });
}
//END:movement