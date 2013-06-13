use std::util;
use contact::contact::Contact;

/**
 * Geometric description of a contact.
 *
 *   - `V`: type of all the contact points, its center and its normal.
 *   - `N`: type of the penetration depth.
 */
pub struct GeometricContact<V, N>
{
  priv world1: V,
  priv world2: V,
  priv center: V,
  priv normal: V,
  priv depth:  N
}

impl<V: Copy + Neg<V>, N: Copy> Contact<V, N> for GeometricContact<V, N>
{
  #[inline(always)]
  fn new(center: &V, normal: &V, depth: &N, world1: &V, world2: &V)
     -> GeometricContact<V, N>
  {
    GeometricContact {
      world1: *world1,
      world2: *world2,
      center: *center,
      normal: *normal,
      depth:  *depth
    }
  }

  #[inline(always)]
  fn flip(&mut self)
  {
    self.normal = -self.normal;
    util::swap(&mut self.world1, &mut self.world2);
  }

  #[inline(always)]
  fn set_center(&mut self, &center: &V)
  { self.center = center; }

  #[inline(always)]
  fn center(&self) -> V
  { self.center }

  #[inline(always)]
  fn set_normal(&mut self, &normal: &V)
  { self.normal = normal; }

  #[inline(always)]
  fn normal(&self) -> V
  { self.normal }

  #[inline(always)]
  fn set_depth(&mut self, &depth: &N)
  { self.depth = depth; }

  #[inline(always)]
  fn depth(&self) -> N
  { self.depth }

  #[inline(always)]
  fn set_world1(&mut self, &world1: &V)
  { self.world1 = world1; }

  #[inline(always)]
  fn world1(&self) -> V
  { self.world1 }

  #[inline(always)]
  fn set_world2(&mut self, &world2: &V)
  { self.world2 = world2; }

  #[inline(always)]
  fn world2(&self) -> V
  { self.world2 }
}
