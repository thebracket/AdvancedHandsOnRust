//START: Tick
use bevy::prelude::*;

// How frequently should the physics tick fire (ms)
const PHYSICS_TICK_TIME: u128 = 33;


#[derive(Default)]
pub struct PhysicsTimer(u128);

#[derive(Event)]
pub struct PhysicsTick;
//END: Tick

//START: physics_clock
pub fn physics_clock(
  mut clock: Local<PhysicsTimer>,
  time: Res<Time>,
  mut on_tick: EventWriter<PhysicsTick>,
) {
  let ms_since_last_call = time.delta().as_millis();
  clock.0 += ms_since_last_call;
  if clock.0 >= PHYSICS_TICK_TIME {
    clock.0 = 0;
    on_tick.write(PhysicsTick);
  }
}
//END: physics_clock


//START: Velocity
#[derive(Component)]
pub struct Velocity(pub Vec3);

impl Default for Velocity {
  fn default() -> Self {
    Self(Vec3::ZERO)
  }
}

impl Velocity {
  pub fn new(x: f32, y: f32, z: f32) -> Self {
    Self(Vec3 { x, y, z })
  }
}
//END: Velocity

//START: Impulse
#[derive(Event)]
pub struct Impulse {
  pub target: Entity,// <callout id="physics_impulse_target" />
  pub amount: Vec3,// <callout id="physics_impulse_amount" />
  pub absolute: bool,// <callout id="physics_impulse_absolute" />
  pub source: i32,// <callout id="physics_impulse_source" />
}
//END: Impulse

//START: sum_impulses
pub fn sum_impulses(
  mut impulses: EventReader<Impulse>,// <callout id="physics_sum_eventreader" />
  mut velocities: Query<&mut Velocity>,// <callout id="physics_sum_query" />
) {
  let mut dedupe_by_source = std::collections::HashMap::new(); // <callout id="physics_sum_dedupe" />
  for impulse in impulses.read() {
    dedupe_by_source.insert(impulse.source, impulse);
  }

  let mut absolute = std::collections::HashSet::new();// <callout id="physics_sum_absolute" />
  for (_, impulse) in dedupe_by_source {
    if let Ok(mut velocity) = velocities.get_mut(impulse.target) {// <callout id="physics_sum_get_mut" />
      if absolute.contains(&impulse.target) {
        continue; // <callout id="physics_sum_stop" />
      }
      if impulse.absolute {
        velocity.0 = impulse.amount;
        absolute.insert(impulse.target);// <callout id="physics_sum_absolute_insert" />
      } else {
        velocity.0 += impulse.amount;
      }
    }
  }
}
//END: sum_impulses

//START: apply_velocity
pub fn apply_velocity(
  mut tick: EventReader<PhysicsTick>,
  mut movement: Query<(&Velocity, &mut Transform)>,
) {
  for _tick in tick.read() {
    movement.iter_mut().for_each(|(velocity, mut transform)| {
      transform.translation += velocity.0;
    });
  }
}
//END: apply_velocity

//START: gravity
#[derive(Component)]
pub struct ApplyGravity;// <callout id="apply_gravity" />

pub fn apply_gravity(
  mut tick: EventReader<PhysicsTick>,
  mut gravity: Query<&mut Velocity, With<ApplyGravity>>,
) {
  for _tick in tick.read() {
    gravity.iter_mut().for_each(|mut velocity| {
      velocity.0.y -= 0.75;
    });
  }
}
//END: gravity