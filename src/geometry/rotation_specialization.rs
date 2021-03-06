#[cfg(feature = "arbitrary")]
use quickcheck::{Arbitrary, Gen};

use std::ops::Neg;
use num::Zero;
use rand::{Rand, Rng};
use alga::general::Real;

use core::{Unit, ColumnVector, SquareMatrix, OwnedSquareMatrix, OwnedColumnVector, Vector3};
use core::dimension::{U1, U2, U3};
use core::storage::{Storage, OwnedStorage};
use core::allocator::{Allocator, OwnedAllocator};

use geometry::{RotationBase, OwnedRotation, UnitComplex};


/*
 *
 * 2D RotationBase matrix.
 *
 */
impl<N, S> RotationBase<N, U2, S>
where N: Real,
      S: OwnedStorage<N, U2, U2>,
      S::Alloc: OwnedAllocator<N, U2, U2, S> {
    /// Builds a 2 dimensional rotation matrix from an angle in radian.
    pub fn new(angle: N) -> Self {
        let (sia, coa) = angle.sin_cos();
        Self::from_matrix_unchecked(SquareMatrix::<N, U2, S>::new(coa, -sia, sia, coa))
    }

    /// Builds a 2 dimensional rotation matrix from an angle in radian wrapped in a 1-dimensional vector.
    ///
    /// Equivalent to `Self::new(axisangle[0])`.
    #[inline]
    pub fn from_scaled_axis<SB: Storage<N, U1, U1>>(axisangle: ColumnVector<N, U1, SB>) -> Self {
        Self::new(axisangle[0])
    }

    /// The rotation matrix required to align `a` and `b` but with its angl.
    ///
    /// This is the rotation `R` such that `(R * a).angle(b) == 0 && (R * a).dot(b).is_positive()`.
    #[inline]
    pub fn rotation_between<SB, SC>(a: &ColumnVector<N, U2, SB>, b: &ColumnVector<N, U2, SC>) -> Self
        where SB: Storage<N, U2, U1>,
              SC: Storage<N, U2, U1> {
        ::convert(UnitComplex::rotation_between(a, b).to_rotation_matrix())
    }

    /// The smallest rotation needed to make `a` and `b` collinear and point toward the same
    /// direction, raised to the power `s`.
    #[inline]
    pub fn scaled_rotation_between<SB, SC>(a: &ColumnVector<N, U2, SB>, b: &ColumnVector<N, U2, SC>, s: N) -> Self
        where SB: Storage<N, U2, U1>,
              SC: Storage<N, U2, U1> {
        ::convert(UnitComplex::scaled_rotation_between(a, b, s).to_rotation_matrix())
    }
}

impl<N, S> RotationBase<N, U2, S>
where N: Real,
      S: Storage<N, U2, U2> {
    /// The rotation angle.
    #[inline]
    pub fn angle(&self) -> N {
        self.matrix()[(1, 0)].atan2(self.matrix()[(0, 0)])
    }

    /// The rotation angle needed to make `self` and `other` coincide.
    #[inline]
    pub fn angle_to<SB: Storage<N, U2, U2>>(&self, other: &RotationBase<N, U2, SB>) -> N {
        self.rotation_to(other).angle()
    }

    /// The rotation matrix needed to make `self` and `other` coincide.
    ///
    /// The result is such that: `self.rotation_to(other) * self == other`.
    #[inline]
    pub fn rotation_to<SB>(&self, other: &RotationBase<N, U2, SB>) -> OwnedRotation<N, U2, SB::Alloc>
        where SB: Storage<N, U2, U2> {
        other * self.inverse()
    }

    /// Raise the quaternion to a given floating power, i.e., returns the rotation with the angle
    /// of `self` multiplied by `n`.
    #[inline]
    pub fn powf(&self, n: N) -> OwnedRotation<N, U2, S::Alloc> {
        OwnedRotation::<_, _, S::Alloc>::new(self.angle() * n)
    }

    /// The rotation angle returned as a 1-dimensional vector.
    #[inline]
    pub fn scaled_axis(&self) -> OwnedColumnVector<N, U1, S::Alloc>
        where S::Alloc: Allocator<N, U1, U1> {
        ColumnVector::<_, U1, _>::new(self.angle())
    }
}

impl<N, S> Rand for RotationBase<N, U2, S>
where N: Real + Rand,
      S: OwnedStorage<N, U2, U2>,
      S::Alloc: OwnedAllocator<N, U2, U2, S> {
    #[inline]
    fn rand<R: Rng>(rng: &mut R) -> Self {
        Self::new(rng.gen())
    }
}

#[cfg(feature="arbitrary")]
impl<N, S> Arbitrary for RotationBase<N, U2, S>
where N: Real + Arbitrary,
      S: OwnedStorage<N, U2, U2> + Send,
      S::Alloc: OwnedAllocator<N, U2, U2, S> {
    #[inline]
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Self::new(N::arbitrary(g))
    }
}


/*
 *
 * 3D RotationBase matrix.
 *
 */
impl<N, S> RotationBase<N, U3, S>
where N: Real,
      S: OwnedStorage<N, U3, U3>,
      S::Alloc: OwnedAllocator<N, U3, U3, S> {

    /// Builds a 3 dimensional rotation matrix from an axis and an angle.
    ///
    /// # Arguments
    ///   * `axisangle` - A vector representing the rotation. Its magnitude is the amount of rotation
    ///   in radian. Its direction is the axis of rotation.
    pub fn new<SB: Storage<N, U3, U1>>(axisangle: ColumnVector<N, U3, SB>) -> Self {
        let (axis, angle) = Unit::new_and_get(axisangle.into_owned());
        Self::from_axis_angle(&axis, angle)
    }

    /// Builds a 3D rotation matrix from an axis scaled by the rotation angle.
    pub fn from_scaled_axis<SB: Storage<N, U3, U1>>(axisangle: ColumnVector<N, U3, SB>) -> Self {
        Self::new(axisangle)
    }

    /// Builds a 3D rotation matrix from an axis and a rotation angle.
    pub fn from_axis_angle<SB>(axis: &Unit<ColumnVector<N, U3, SB>>, angle: N) -> Self
        where SB: Storage<N, U3, U1> {
        if angle.is_zero() {
            Self::identity()
        }
        else {
            let ux         = axis.as_ref()[0];
            let uy         = axis.as_ref()[1];
            let uz         = axis.as_ref()[2];
            let sqx        = ux * ux;
            let sqy        = uy * uy;
            let sqz        = uz * uz;
            let (sin, cos) = angle.sin_cos();
            let one_m_cos  = N::one() - cos;

            Self::from_matrix_unchecked(
                SquareMatrix::<N, U3, S>::new(
                    (sqx + (N::one() - sqx) * cos),
                    (ux * uy * one_m_cos - uz * sin),
                    (ux * uz * one_m_cos + uy * sin),

                    (ux * uy * one_m_cos + uz * sin),
                    (sqy + (N::one() - sqy) * cos),
                    (uy * uz * one_m_cos - ux * sin),

                    (ux * uz * one_m_cos - uy * sin),
                    (uy * uz * one_m_cos + ux * sin),
                    (sqz + (N::one() - sqz) * cos)))
        }
    }

    /// Creates a new rotation from Euler angles.
    ///
    /// The primitive rotations are applied in order: 1 roll − 2 pitch − 3 yaw.
    pub fn from_euler_angles(roll: N, pitch: N, yaw: N) -> Self {
        let (sr, cr) = roll.sin_cos();
        let (sp, cp) = pitch.sin_cos();
        let (sy, cy) = yaw.sin_cos();

        Self::from_matrix_unchecked(
            SquareMatrix::<N, U3, S>::new(
                cy * cp, cy * sp * sr - sy * cr, cy * sp * cr + sy * sr,
                sy * cp, sy * sp * sr + cy * cr, sy * sp * cr - cy * sr,
                -sp,     cp * sr,                cp * cr)
            )
    }

    /// Creates a rotation that corresponds to the local frame of an observer standing at the
    /// origin and looking toward `dir`.
    ///
    /// It maps the view direction `dir` to the positive `z` axis.
    ///
    /// # Arguments
    ///   * dir - The look direction, that is, direction the matrix `z` axis will be aligned with.
    ///   * up - The vertical direction. The only requirement of this parameter is to not be
    ///   collinear
    ///   to `dir`. Non-collinearity is not checked.
    #[inline]
    pub fn new_observer_frame<SB, SC>(dir: &ColumnVector<N, U3, SB>, up: &ColumnVector<N, U3, SC>) -> Self
    where SB: Storage<N, U3, U1>,
          SC: Storage<N, U3, U1> {
        let zaxis = dir.normalize();
        let xaxis = up.cross(&zaxis).normalize();
        let yaxis = zaxis.cross(&xaxis).normalize();

        Self::from_matrix_unchecked(SquareMatrix::<N, U3, S>::new(
                xaxis.x, yaxis.x, zaxis.x,
                xaxis.y, yaxis.y, zaxis.y,
                xaxis.z, yaxis.z, zaxis.z))
    }


    /// Builds a right-handed look-at view matrix without translation.
    ///
    /// This conforms to the common notion of right handed look-at matrix from the computer
    /// graphics community.
    ///
    /// # Arguments
    ///   * eye - The eye position.
    ///   * target - The target position.
    ///   * up - A vector approximately aligned with required the vertical axis. The only
    ///   requirement of this parameter is to not be collinear to `target - eye`.
    #[inline]
    pub fn look_at_rh<SB, SC>(dir: &ColumnVector<N, U3, SB>, up: &ColumnVector<N, U3, SC>) -> Self
    where SB: Storage<N, U3, U1>,
          SC: Storage<N, U3, U1> {
        Self::new_observer_frame(&dir.neg(), up).inverse()
    }

    /// Builds a left-handed look-at view matrix without translation.
    ///
    /// This conforms to the common notion of left handed look-at matrix from the computer
    /// graphics community.
    ///
    /// # Arguments
    ///   * eye - The eye position.
    ///   * target - The target position.
    ///   * up - A vector approximately aligned with required the vertical axis. The only
    ///   requirement of this parameter is to not be collinear to `target - eye`.
    #[inline]
    pub fn look_at_lh<SB, SC>(dir: &ColumnVector<N, U3, SB>, up: &ColumnVector<N, U3, SC>) -> Self
    where SB: Storage<N, U3, U1>,
          SC: Storage<N, U3, U1> {
            Self::new_observer_frame(dir, up).inverse()
    }

    /// The rotation matrix required to align `a` and `b` but with its angl.
    ///
    /// This is the rotation `R` such that `(R * a).angle(b) == 0 && (R * a).dot(b).is_positive()`.
    #[inline]
    pub fn rotation_between<SB, SC>(a: &ColumnVector<N, U3, SB>, b: &ColumnVector<N, U3, SC>) -> Option<Self>
        where SB: Storage<N, U3, U1>,
              SC: Storage<N, U3, U1> {
        Self::scaled_rotation_between(a, b, N::one())
    }

    /// The smallest rotation needed to make `a` and `b` collinear and point toward the same
    /// direction, raised to the power `s`.
    #[inline]
    pub fn scaled_rotation_between<SB, SC>(a: &ColumnVector<N, U3, SB>, b: &ColumnVector<N, U3, SC>, n: N)
        -> Option<Self>
        where SB: Storage<N, U3, U1>,
              SC: Storage<N, U3, U1> {
        // FIXME: code duplication with RotationBase.
        if let (Some(na), Some(nb)) = (a.try_normalize(N::zero()), b.try_normalize(N::zero())) {
            let c = na.cross(&nb);

            if let Some(axis) = Unit::try_new(c, N::default_epsilon()) {
                return Some(Self::from_axis_angle(&axis, na.dot(&nb).acos() * n))
            }

            // Zero or PI.
            if na.dot(&nb) < N::zero() {
                // PI
                //
                // The rotation axis is undefined but the angle not zero. This is not a
                // simple rotation.
                return None;
            }
        }

        Some(Self::identity())
    }
}

impl<N, S> RotationBase<N, U3, S>
where N: Real,
      S: Storage<N, U3, U3> {
    /// The rotation angle.
    #[inline]
    pub fn angle(&self) -> N {
        ((self.matrix()[(0, 0)] + self.matrix()[(1, 1)] + self.matrix()[(2, 2)] - N::one()) / ::convert(2.0)).acos()
    }
}

impl<N, S> RotationBase<N, U3, S>
where N: Real,
      S: Storage<N, U3, U3>,
      S::Alloc: Allocator<N, U3, U1> {
    /// The rotation axis. Returns `None` if the rotation angle is zero or PI.
    #[inline]
    pub fn axis(&self) -> Option<Unit<OwnedColumnVector<N, U3, S::Alloc>>> {
        let axis = OwnedColumnVector::<N, U3, S::Alloc>::new(
            self.matrix()[(2, 1)] - self.matrix()[(1, 2)],
            self.matrix()[(0, 2)] - self.matrix()[(2, 0)],
            self.matrix()[(1, 0)] - self.matrix()[(0, 1)]);

        Unit::try_new(axis, N::default_epsilon())
    }

    /// The rotation axis multiplied by the rotation angle.
    #[inline]
    pub fn scaled_axis(&self) -> OwnedColumnVector<N, U3, S::Alloc> {
        if let Some(axis) = self.axis() {
            axis.unwrap() * self.angle()
        }
        else {
            ColumnVector::zero()
        }
    }

    /// The rotation angle needed to make `self` and `other` coincide.
    #[inline]
    pub fn angle_to<SB: Storage<N, U3, U3>>(&self, other: &RotationBase<N, U3, SB>) -> N {
        self.rotation_to(other).angle()
    }

    /// The rotation matrix needed to make `self` and `other` coincide.
    ///
    /// The result is such that: `self.rotation_to(other) * self == other`.
    #[inline]
    pub fn rotation_to<SB>(&self, other: &RotationBase<N, U3, SB>) -> OwnedRotation<N, U3, SB::Alloc>
        where SB: Storage<N, U3, U3> {
        other * self.inverse()
    }

    /// Raise the quaternion to a given floating power, i.e., returns the rotation with the same
    /// axis as `self` and an angle equal to `self.angle()` multiplied by `n`.
    #[inline]
    pub fn powf(&self, n: N) -> OwnedRotation<N, U3, S::Alloc> {
        if let Some(axis) = self.axis() {
            OwnedRotation::<_, _, S::Alloc>::from_axis_angle(&axis, self.angle() * n)
        }
        else if self.matrix()[(0, 0)] < N::zero() {
            let minus_id = OwnedSquareMatrix::<N, U3, S::Alloc>::from_diagonal_element(-N::one());
            OwnedRotation::<_, _, S::Alloc>::from_matrix_unchecked(minus_id)
        }
        else {
            OwnedRotation::<_, _, S::Alloc>::identity()
        }
    }
}

impl<N, S> Rand for RotationBase<N, U3, S>
where N: Real + Rand,
      S: OwnedStorage<N, U3, U3>,
      S::Alloc: OwnedAllocator<N, U3, U3, S> +
                Allocator<N, U3, U1> {
    #[inline]
    fn rand<R: Rng>(rng: &mut R) -> Self {
        Self::new(Vector3::rand(rng))
    }
}

#[cfg(feature="arbitrary")]
impl<N, S> Arbitrary for RotationBase<N, U3, S>
where N: Real + Arbitrary,
      S: OwnedStorage<N, U3, U3> + Send,
      S::Alloc: OwnedAllocator<N, U3, U3, S> +
                Allocator<N, U3, U1> {
    #[inline]
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Self::new(Vector3::arbitrary(g))
    }
}
