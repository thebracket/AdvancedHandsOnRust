//START: import
use bevy::{prelude::*, platform::collections::HashSet};
use super::rect2d::Rect2D;
//END: import

//START: quadtree
#[derive(Debug, Resource)]
pub struct StaticQuadTree {
  nodes: Vec<StaticQuadTreeNode>,
}

#[derive(Debug)]
pub struct StaticQuadTreeNode {
  bounds: Rect2D,
  children: Option<[usize; 4]>,
}
//END: quadtree

//START: staticquadtreenew
impl StaticQuadTree {
  pub fn new(screen_size: Vec2, max_depth: usize) -> Self {
    // Make the container
    let mut nodes = Vec::new();

    // Create the top-level, always the whole screen
    let half = screen_size / 2.0;
    let top = StaticQuadTreeNode {
      bounds: Rect2D::new(
        Vec2::new(0.0 - half.x, 0.0 - half.y),
        Vec2::new(half.x, half.y),
      ),
      children: None,
    };
    nodes.push(top);

    Self::subdivide(&mut nodes, 0, 1, max_depth);

    Self { nodes }
  }
  //END: staticquadtreenew

//START: smallestnode
  pub fn smallest_node(&self, target: &Rect2D) -> usize {
    let mut current_index = 0;

    #[allow(clippy::while_let_loop)]
    loop {
      if let Some(children) = self.nodes[current_index].children {
        let matches: Vec<usize> = children
          .iter()
          .filter_map(|child| {
            if self.nodes[*child].bounds.intersect(target) {
              Some(*child)
            } else {
              None
            }
          })
          .collect();

        if matches.len() == 1 {
          current_index = matches[0];
        } else {
          break;
        }
      } else {
        break;
      }
    }

    current_index
  }
//END: smallestnode

//START: staticquadtreesubdivide
  fn subdivide(
    nodes: &mut Vec<StaticQuadTreeNode>,
    index: usize,
    depth: usize,
    max_depth: usize,
  ) {
    let mut children = nodes[index].bounds.quadrants();
    let child_index = [
      nodes.len(),
      nodes.len() + 1,
      nodes.len() + 2,
      nodes.len() + 3,
    ];
    nodes[index].children = Some(child_index);
    children.drain(0..4).for_each(|quad| {
      nodes.push(StaticQuadTreeNode {
        bounds: quad,
        children: None,
      })
    });

    if depth < max_depth {
      for index in child_index {
        Self::subdivide(nodes, index, depth + 1, max_depth);
      }
    }
  }
  //END: staticquadtreesubdivide

//START: staticquadtreeintersecting_nodes
  pub fn intersecting_nodes(&self, target: &Rect2D) -> HashSet<usize> {
    let mut result = HashSet::new();
    self.intersect(0, &mut result, target);
    result
  }

  fn intersect(
    &self,
    index: usize,
    result: &mut HashSet<usize>,
    target: &Rect2D,
  ) {
    if self.nodes[index].bounds.intersect(target) {
      result.insert(index);
      if let Some(children) = &self.nodes[index].children {
        for child in children {
          self.intersect(*child, result, target);
        }
      }
    }
  }
}
//END: staticquadtreeintersecting_nodes