use std::num::{One, Zero};
use std::rand::{Rand, Rng, RngUtil};
use std::cmp::ApproxEq;
use traits::dim::Dim;
use traits::inv::Inv;
use traits::rotation::{Rotation, Rotate};
use traits::translation::{Translation, Translate};
use Ts = traits::transformation::Transform;
use traits::transformation::{Transformation};
use traits::rlmul::{RMul, LMul};
use traits::homogeneous::{ToHomogeneous, FromHomogeneous};
use traits::column::Column;
use adaptors::rotmat::Rotmat;
use vec::Vec3;
use mat::Mat3;

/// Matrix-Vector wrapper used to represent a matrix multiplication followed by a translation.
/// Usually, a matrix in homogeneous coordinate is used to be able to apply an affine transform with
/// a translation to a vector. This is weird because it makes a `n`-dimentional transformation be
/// an `n + 1`-matrix. Using the `Transform` wrapper avoid homogeneous coordinates by having the
/// translation separate from the other transformations. This is particularity useful when the
/// underlying transform is a rotation (see `Rotmat`): this makes inversion much faster than
/// inverting the homogeneous matrix itself.
#[deriving(Eq, ToStr, Clone)]
pub struct Transform<V, M> {
    priv submat   : M,
    priv subtrans : V
}

// FIXME: this should be Trasform<V, M>
impl<V, M> Transform<V, M> {
    /// Builds a new transform from a matrix and a vector.
    #[inline]
    pub fn new(trans: V, mat: M) -> Transform<V, M> {
        Transform {
            submat:   mat,
            subtrans: trans
        }
    }
}

impl<V: Clone, M: Clone> Transform<V, M> {
    /// Gets a copy of the internal matrix.
    #[inline]
    pub fn submat(&self) -> M {
        self.submat.clone()
    }

    /// Gets a copy of the internal translation.
    #[inline]
    pub fn subtrans(&self) -> V {
        self.subtrans.clone()
    }
}

impl<N: Clone + Num + Algebraic> Transform<Vec3<N>, Rotmat<Mat3<N>>> {
    /// Reorient and translate this transformation such that its local `x` axis points to a given
    /// direction.  Note that the usually known `look_at` function does the same thing but with the
    /// `z` axis. See `look_at_z` for that.
    ///
    /// # Arguments
    ///   * eye - The new translation of the transformation.
    ///   * at - The point to look at. `at - eye` is the direction the matrix `x` axis will be
    ///   aligned with
    ///   * up - Vector pointing `up`. The only requirement of this parameter is to not be colinear
    ///   with `at`. Non-colinearity is not checked.
    pub fn look_at(&mut self, eye: &Vec3<N>, at: &Vec3<N>, up: &Vec3<N>) {
        self.submat.look_at(&(*at - *eye), up);
        self.subtrans = eye.clone();
    }

    /// Reorient and translate this transformation such that its local `z` axis points to a given
    /// direction. 
    ///
    /// # Arguments
    ///   * eye - The new translation of the transformation.
    ///   * at - The point to look at. `at - eye` is the direction the matrix `x` axis will be
    ///   aligned with
    ///   * up - Vector pointing `up`. The only requirement of this parameter is to not be colinear
    ///   with `at`. Non-colinearity is not checked.
    pub fn look_at_z(&mut self, eye: &Vec3<N>, at: &Vec3<N>, up: &Vec3<N>) {
        self.submat.look_at_z(&(*at - *eye), up);
        self.subtrans = eye.clone();
    }
}

impl<M: Dim, V> Dim for Transform<V, M> {
    #[inline]
    fn dim(_: Option<Transform<V, M>>) -> uint {
        Dim::dim(None::<M>)
    }
}

impl<M: One, V: Zero> One for Transform<V, M> {
    #[inline]
    fn one() -> Transform<V, M> {
        Transform {
            submat: One::one(), subtrans: Zero::zero()
        }
    }
}

impl<M: Zero, V: Zero> Zero for Transform<V, M> {
    #[inline]
    fn zero() -> Transform<V, M> {
        Transform {
            submat: Zero::zero(), subtrans: Zero::zero()
        }
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.submat.is_zero() && self.subtrans.is_zero()
    }
}

impl<M: RMul<V> + Mul<M, M>, V: Add<V, V>>
Mul<Transform<V, M>, Transform<V, M>> for Transform<V, M> {
    #[inline]
    fn mul(&self, other: &Transform<V, M>) -> Transform<V, M> {
        Transform {
            submat:   self.submat * other.submat,
            subtrans: self.subtrans + self.submat.rmul(&other.subtrans)
        }
    }
}

impl<M: RMul<V>, V: Add<V, V>> RMul<V> for Transform<V, M> {
    #[inline]
    fn rmul(&self, other: &V) -> V {
        self.submat.rmul(other) + self.subtrans
    }
}

impl<M: LMul<V>, V: Add<V, V>> LMul<V> for Transform<V, M> {
    #[inline]
    fn lmul(&self, other: &V) -> V {
        self.submat.lmul(other) + self.subtrans
    }
}

impl<M: Clone, V: Translation<V>> Translation<V> for Transform<V, M> {
    #[inline]
    fn translation(&self) -> V {
        self.subtrans.translation()
    }

    #[inline]
    fn inv_translation(&self) -> V {
        self.subtrans.inv_translation()
    }

    #[inline]
    fn translate_by(&mut self, t: &V) {
        self.subtrans.translate_by(t)
    }

    #[inline]
    fn translated(&self, t: &V) -> Transform<V, M> {
        Transform::new(self.subtrans.translated(t), self.submat.clone())
    }

    #[inline]
    fn set_translation(&mut self, t: V) {
        self.subtrans.set_translation(t)
    }
}

impl<V, M: Translate<V>, _0> Translate<V> for Transform<_0, M> {
    #[inline]
    fn translate(&self, v: &V) -> V {
        self.submat.translate(v)
    }

    #[inline]
    fn inv_translate(&self, v: &V) -> V {
        self.submat.inv_translate(v)
    }
}

impl<M: Rotation<AV> + RMul<V> + One, V, AV>
Rotation<AV> for Transform<V, M> {
    #[inline]
    fn rotation(&self) -> AV {
        self.submat.rotation()
    }

    #[inline]
    fn inv_rotation(&self) -> AV {
        self.submat.inv_rotation()
    }


    #[inline]
    fn rotate_by(&mut self, rot: &AV) {
        // FIXME: this does not seem opitmal
        let mut delta: M = One::one();
        delta.rotate_by(rot);
        self.submat.rotate_by(rot);
        self.subtrans = delta.rmul(&self.subtrans);
    }

    #[inline]
    fn rotated(&self, rot: &AV) -> Transform<V, M> {
        // FIXME: this does not seem opitmal
        let _1: M = One::one();
        let delta = _1.rotated(rot);

        Transform::new(delta.rmul(&self.subtrans), self.submat.rotated(rot))
    }

    #[inline]
    fn set_rotation(&mut self, rot: AV) {
        // FIXME: should the translation be changed too?
        self.submat.set_rotation(rot)
    }
}

impl<V, M: Rotate<V>, _0> Rotate<V> for Transform<_0, M> {
    #[inline]
    fn rotate(&self, v: &V) -> V {
        self.submat.rotate(v)
    }

    #[inline]
    fn inv_rotate(&self, v: &V) -> V {
        self.submat.inv_rotate(v)
    }
}

impl<M: Inv + RMul<V> + Mul<M, M> + Clone, V: Add<V, V> + Neg<V> + Clone>
Transformation<Transform<V, M>> for Transform<V, M> {
    fn transformation(&self) -> Transform<V, M> {
        self.clone()
    }

    fn inv_transformation(&self) -> Transform<V, M> {
        // FIXME: fail or return a Some<Transform<V, M>> ?
        match self.inverse() {
            Some(t) => t,
            None    => fail!("This transformation was not inversible.")
        }
    }

    fn transform_by(&mut self, other: &Transform<V, M>) {
        *self = other * *self
    }

    fn transformed(&self, t: &Transform<V, M>) -> Transform<V, M> {
        t * *self
    }

    fn set_transformation(&mut self, t: Transform<V, M>) {
        *self = t
    }
}

impl<M: Ts<V>, V: Add<V, V> + Sub<V, V>>
Ts<V> for Transform<V, M> {
    #[inline]
    fn transform(&self, v: &V) -> V {
        self.submat.transform(v) + self.subtrans
    }

    #[inline]
    fn inv_transform(&self, v: &V) -> V {
        self.submat.inv_transform(&(v - self.subtrans))
    }
}

impl<M: Inv + RMul<V> + Clone, V: Neg<V> + Clone>
Inv for Transform<V, M> {
    #[inline]
    fn inplace_inverse(&mut self) -> bool {
        if !self.submat.inplace_inverse() {
            false
        }
        else {
            self.subtrans = self.submat.rmul(&-self.subtrans);
            true
        }
    }

    #[inline]
    fn inverse(&self) -> Option<Transform<V, M>> {
        let mut res = self.clone();

        if res.inplace_inverse() {
            Some(res)
        }
        else {
            None
        }
    }
}

impl<M: ToHomogeneous<M2>, M2: Dim + Column<V2>, V: ToHomogeneous<V2> + Clone, V2>
ToHomogeneous<M2> for Transform<V, M> {
    fn to_homogeneous(&self) -> M2 {
        let mut res = self.submat.to_homogeneous();

        // copy the translation
        let dim = Dim::dim(None::<M2>);

        res.set_column(dim - 1, self.subtrans.to_homogeneous());

        res
    }
}

impl<M: Column<V> + Dim, M2: FromHomogeneous<M>, V>
FromHomogeneous<M> for Transform<V, M2> {
    fn from(m: &M) -> Transform<V, M2> {
        Transform::new(m.column(Dim::dim(None::<M>) - 1), FromHomogeneous::from(m))
    }
}

impl<N: ApproxEq<N>, M:ApproxEq<N>, V:ApproxEq<N>>
ApproxEq<N> for Transform<V, M> {
    #[inline]
    fn approx_epsilon() -> N {
        fail!("approx_epsilon is broken since rust revision 8693943676487c01fa09f5f3daf0df6a1f71e24d.")
        // ApproxEq::<N>::approx_epsilon()
    }

    #[inline]
    fn approx_eq(&self, other: &Transform<V, M>) -> bool {
        self.submat.approx_eq(&other.submat) &&
            self.subtrans.approx_eq(&other.subtrans)
    }

    #[inline]
    fn approx_eq_eps(&self, other: &Transform<V, M>, epsilon: &N) -> bool {
        self.submat.approx_eq_eps(&other.submat, epsilon) &&
            self.subtrans.approx_eq_eps(&other.subtrans, epsilon)
    }
}

impl<M: Rand, V: Rand> Rand for Transform<V, M> {
    #[inline]
    fn rand<R: Rng>(rng: &mut R) -> Transform<V, M> {
        Transform::new(rng.gen(), rng.gen())
    }
}