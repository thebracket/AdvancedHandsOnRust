use bevy::{prelude::*, platform::collections::HashMap, log};

//START: AnimationOption
pub enum AnimationOption {
    None,//<callout id="anim_opt_none" />
    NextFrame, //<callout id="anim_opt_next" />
    GoToFrame(usize),//<callout id="anim_opt_goto" />
    SwitchToAnimation(String),//<callout id="anim_opt_switch" />
    PlaySound(String),//<callout id="anim_opt_sound" />
}
//END: AnimationOption

//START: AnimationFrame
pub struct AnimationFrame {
    sprite_index: usize,//<callout id="animation_frame_sprite_index" />
    delay_ms: u128,//<callout id="animation_frame_delay" />
    action: Vec<AnimationOption>,//<callout id="animation_frame_options" />
}

impl AnimationFrame {
    pub fn new(//<callout id="animation_frame_constructor" />
        sprite_index: usize, 
        delay_ms: u128, 
        action: Vec<AnimationOption>
    ) -> Self {
        Self { sprite_index, delay_ms, action }
    }
}
//END: AnimationFrame

//START: PerFrameAnimation
pub struct PerFrameAnimation {
    pub frames: Vec<AnimationFrame>,
}

impl PerFrameAnimation {
    pub fn new(frames: Vec<AnimationFrame>) -> Self {
        Self { frames }
    }
}
//END: PerFrameAnimation

//START: Animations
#[derive(Resource)]
pub struct Animations(HashMap<String, PerFrameAnimation>);

impl Animations {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn with_animation<S: ToString>(
        mut self, 
        tag: S, 
        animation: PerFrameAnimation
    ) -> Self
    {
        self.0.insert(
            tag.to_string(),
            animation
        );
        self
    }
}
//END: Animations

//START: AnimationCycle
#[derive(Component)]
pub struct AnimationCycle {
    animation_tag: String,// <callout id="animation_cycle_tag" />
    current_frame: usize,// <callout id="animation_cycle_frame" />
    timer: u128,// <callout id="animation_cycle_timer" />
}

impl AnimationCycle {
    pub fn new<S: ToString>(tag: S) -> Self {// <callout id="animation_cycle_constructor" />
        Self {
            animation_tag: tag.to_string(),
            current_frame: 0,
            timer: 0,
        }
    }

    pub fn switch<S: ToString>(&mut self, new: S) {// <callout id="animation_cycle_switch" />
        let new = new.to_string();
        if new != self.animation_tag {// <callout id="animation_cycle_switch_if" />
            self.animation_tag = new;
            self.current_frame = 0;
            self.timer = 0;
        }
    }
}
//END: AnimationCycle

//START: cycle_animations1
pub fn cycle_animations(
    animations: Res<Animations>,// <callout id="cycle_animations_anim_resource" />
    mut animated: Query<(&mut AnimationCycle, &mut Sprite)>,// <callout id="cycle_animations_animated" />
    time: Res<Time>,// <callout id="cycle_animations_anim_time" />
    assets: Res<crate::AssetStore>,// <callout id="cycle_animations_anim_assets" />
    mut commands: Commands,
    loaded_assets: Res<crate::LoadedAssets>,
) {//END: cycle_animations1
//START: cycle_animations2
    let ms_since_last_call = time.delta().as_millis();// <callout id="cycle_animations_get_time" />
    animated.iter_mut().for_each(|(mut animation, mut sprite)| {// <callout id="cycle_animations_run_query" />
        animation.timer += ms_since_last_call;// <callout id="cycle_animations_add_time" />
        if let Some(cycle) = animations.0.get(&animation.animation_tag) {// <callout id="cycle_animations_get_cycle" />
            let current_frame = &cycle.frames[animation.current_frame];// <callout id="cycle_animations_get_current_frame_ref" />
            if animation.timer > current_frame.delay_ms {// <callout id="cycle_animations_is_it_time" />
                animation.timer = 0;// <callout id="cycle_animations_reset_time" />
                for action in current_frame.action.iter() { // <callout id="cycle_animations_iter_actions" />
                    match action {
                        AnimationOption::None => {},
                        AnimationOption::NextFrame => { // <callout id="cycle_animations_next_frame" />
                            animation.current_frame += 1;
                        }
                        AnimationOption::GoToFrame(frame) => { // <callout id="cycle_animations_goto" />
                            animation.current_frame = *frame;
                        }
                        AnimationOption::SwitchToAnimation(new) => { // <callout id="cycle_animations_switch" />
                            animation.animation_tag = new.to_string();
                            animation.current_frame = 0;
                        }
                        AnimationOption::PlaySound(tag) => { // <callout id="cycle_animations_play_sound" />
                            assets.play(tag, &mut commands, &loaded_assets);
                        }
                    }
                    if let Some(ta) = &mut sprite.texture_atlas {
                       ta.index = cycle
                        .frames[animation.current_frame]
                        .sprite_index; // <callout id="cycle_animations_new_frame" />
                    }
            }
            }
        } else {
            log::warn!("Animation Cycle [{}] not found!", 
               animation.animation_tag);
        }
    });
}
//END: cycle_animations2

//START: animation_macro
#[macro_export]
macro_rules! spawn_animated_sprite {
   ($assets:expr, $commands:expr, $index:expr, $x:expr, $y:expr, $z:expr,
      $animation_name:expr, $($component:expr),*) => 
  {
      let Some((img, atlas)) = $assets.get_atlas_handle($index) else { panic!() };
      $commands.spawn((
         Sprite::from_atlas_image(img.clone(), TextureAtlas {
             layout: atlas.clone(),
             index: 0,
         }),
         Transform::from_xyz($x, $y, $z),
         AnimationCycle::new($animation_name),
      ))
      $(
       .insert($component)
      )*;
  }
}
  //END: animation_macro

//START: ContinualParallax
#[derive(Component)]
pub struct ContinualParallax {
  image_width: f32, // <callout id="parallax_image_width" />
  move_every_ms: u128, // <callout id="parallax_move_every" />
  scroll_speed: Vec2, // <callout id="parallax_scroll_speed" />
  timer: u128, // <callout id="parallax_timer" />
}

impl ContinualParallax {
  pub fn new(image_width: f32, move_every_ms: u128, scroll_speed: Vec2) 
  -> Self {
    Self {
      image_width, move_every_ms, scroll_speed, timer: 0
    }
  }
}
//END: ContinualParallax

//START: continual_parallax
pub fn continual_parallax(
  mut animated: Query<(&mut ContinualParallax, &mut Transform)>,
  time: Res<Time>,
) {
  let ms_since_last_call = time.delta().as_millis();
  animated.iter_mut().for_each(|(mut parallax, mut transform)| {
    parallax.timer += ms_since_last_call;
    if parallax.timer >= parallax.move_every_ms {
      parallax.timer = 0;
      transform.translation.x -= parallax.scroll_speed.x;
      transform.translation.y -= parallax.scroll_speed.y;
      if transform.translation.x <= (0.0 - parallax.image_width) {
        transform.translation.x = parallax.image_width;
      }
    }
  });
}
//END: continual_parallax