//START: includes
mod aabb;
mod rect2d;
mod static_quadtree;
pub use aabb::AxisAlignedBoundingBox;
pub use rect2d::Rect2D;
pub use static_quadtree::*;
use bevy::{prelude::*, platform::collections::HashMap};
use std::marker::PhantomData;
//END: includes

//START: message
#[derive(Event)]
pub struct OnCollision<A, B>// <callout id="co.quadtree.oncollision.a_b" />
where// <callout id="co.quadtree.oncollision.a_b_where" />
  A: Component,
  B: Component,
{
  pub entity_a: Entity,// <callout id="co.quadtree.oncollision.a_b_entity" />
  pub entity_b: Entity,
  marker: PhantomData<(A, B)>,// <callout id="co.quadtree.phantomdata" />
}
//END: message

//START: check_collisions
pub fn check_collisions<A, B>(// <callout id="co.quadtree.check_collisions.a_b" />
  quad_tree: Res<StaticQuadTree>,// <callout id="co.quadtree.check_collisions.qtresource" />
  query_a: Query<(Entity, &Transform, &AxisAlignedBoundingBox), With<A>>,// <callout id="co.quadtree.check_collisions.with_a" />
  query_b: Query<(Entity, &Transform, &AxisAlignedBoundingBox), With<B>>,
  mut sender: EventWriter<OnCollision<A, B>>,// <callout id="co.quadtree.check_collisions.sender" />
) where
  A: Component,
  B: Component,
{
  let mut spatial_index: HashMap<usize, Vec<(Entity, Rect2D)>> =// <callout id="co.quadtree.check_collisions.same_as_before" />
    HashMap::new();

  query_b.iter().for_each(|(entity, transform, bbox)| {
    let bbox = bbox.as_rect(transform.translation.truncate());
    let in_node = quad_tree.smallest_node(&bbox);
    if let Some(contents) = spatial_index.get_mut(&in_node) {
      contents.push((entity, bbox));
    } else {
      spatial_index.insert(in_node, vec![(entity, bbox)]);
    }
  });

  query_a.iter().for_each(|(entity_a, transform_a, bbox_a)| {
    let bbox_a = bbox_a.as_rect(transform_a.translation.truncate());
    for node in quad_tree.intersecting_nodes(&bbox_a) {
      if let Some(contents) = spatial_index.get(&node) {
        for (entity_b, bbox_b) in contents {
          if entity_a != *entity_b && bbox_a.intersect(bbox_b) {
            sender.write(OnCollision {// <callout id="co.quadtree.check_collisions.send_message" />
              entity_a,
              entity_b: *entity_b,
              marker: PhantomData,// <callout id="co.quadtree.check_collisions.make_phantom" />
            });
          }
        }
      }
    }
  });
}
//END: check_collisions