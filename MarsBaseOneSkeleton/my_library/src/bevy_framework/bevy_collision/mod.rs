//START: Imports
mod aabb;
mod rect2d;
mod static_quadtree;
pub use aabb::AxisAlignedBoundingBox;
pub use rect2d::Rect2D;
pub use static_quadtree::*;
use bevy::{prelude::*, platform::collections::HashMap};
use std::marker::PhantomData;
//START_HIGHLIGHT
use crate::PhysicsPosition;
//END_HIGHLIGHT
//END: Imports

#[derive(Event)]
pub struct OnCollision<A, B>
where
  A: Component,
  B: Component,
{
  pub entity_a: Entity,
  pub entity_b: Entity,
  marker: PhantomData<(A, B)>,
}

//START: CheckCollisions1
pub fn check_collisions<A, B>(
  quad_tree: Res<StaticQuadTree>,
  //START_HIGHLIGHT
  query_a: Query<
    (Entity, &PhysicsPosition, &AxisAlignedBoundingBox), 
    With<A>
  >,
  query_b: Query<
    (Entity, &PhysicsPosition, &AxisAlignedBoundingBox), 
    With<B>
  >,
  //END_HIGHLIGHT
  mut sender: EventWriter<OnCollision<A, B>>,
) where
  A: Component,
  B: Component,
//END: CheckCollisions1
{
  let mut spatial_index: HashMap<usize, Vec<(Entity, Rect2D)>> =
    HashMap::new();

//START: CheckCollisions2
  query_b.iter().for_each(|(entity, transform, bbox)| {
    //START_HIGHLIGHT
    let bbox = bbox.as_rect(transform.end_frame);
    //END_HIGHLIGHT
    let in_node = quad_tree.smallest_node(&bbox);
    if let Some(contents) = spatial_index.get_mut(&in_node) {
      contents.push((entity, bbox));
    } else {
      spatial_index.insert(in_node, vec![(entity, bbox)]);
    }
  });

  query_a.iter().for_each(|(entity_a, transform_a, bbox_a)| {
    //START_HIGHLIGHT
    let bbox_a = bbox_a.as_rect(transform_a.end_frame);
    //END_HIGHLIGHT
    for node in quad_tree.intersecting_nodes(&bbox_a) {
      if let Some(contents) = spatial_index.get(&node) {
        for (entity_b, bbox_b) in contents {
          if entity_a != *entity_b && bbox_a.intersect(bbox_b) {
            sender.write(OnCollision {
              entity_a,
              entity_b: *entity_b,
              marker: PhantomData,
            });
          }
        }
      }
    }
  });
  //END: CheckCollisions2
}
