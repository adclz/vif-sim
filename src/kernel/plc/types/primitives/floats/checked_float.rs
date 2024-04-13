use crate::container::error::error::Stop;
use crate::error;

pub trait CheckedFloat<Rhs = Self> {
    fn checked_add(self, rhs: Rhs) -> Option<Rhs>;
    fn checked_sub(self, rhs: Rhs) -> Option<Rhs>;
    fn checked_mul(self, rhs: Rhs) -> Option<Rhs>;
    fn checked_div(self, rhs: Rhs) -> Option<Rhs>;
    fn checked_rem(self, rhs: Rhs) -> Option<Rhs>;
    // Trig
    fn checked_cos(self) -> Option<Rhs>;
    fn checked_sin(self) -> Option<Rhs>;
    fn checked_tan(self) -> Option<Rhs>;
    fn checked_acos(self) -> Option<Rhs>;
    fn checked_asin(self) -> Option<Rhs>;
    fn checked_atan(self) -> Option<Rhs>;
    fn checked_exp(self) -> Option<Rhs>;
    fn checked_ln(self) -> Option<Rhs>;

    // Parts
    fn checked_fract(self) -> Option<Rhs>;
    fn checked_trunc(self) -> Option<Rhs>;
    fn checked_floor(self) -> Option<Rhs>;
    fn checked_ceil(self) -> Option<Rhs>;
    fn checked_sqrt(self) -> Option<Rhs>;
    fn checked_sqr(self) -> Option<Rhs>;

}

macro_rules! impl_checked_float {
    ($ty: ty) => {
        impl CheckedFloat for $ty {
            fn checked_add(self, v: $ty) -> Option<$ty> {
                let result = self + v;
                match !result.is_infinite() && !result.is_nan() && !result.is_subnormal() {
                    true => Some(result),
                    false => None
                }
            }

            fn checked_sub(self, v: $ty) -> Option<$ty> {
                let result = self - v;
                match !result.is_infinite() && !result.is_nan() && !result.is_subnormal() {
                    true => Some(result),
                    false => None
                }
            }

            fn checked_mul(self, v: $ty) -> Option<$ty> {
                let result = self * v;
                match !result.is_infinite() && !result.is_nan() && !result.is_subnormal() {
                    true => Some(result),
                    false => None
                }
            }

            fn checked_div(self, v: $ty) -> Option<$ty> {
                let result = self / v;
                match !result.is_infinite() && !result.is_nan() && !result.is_subnormal() {
                    true => Some(result),
                    false => None
                }
            }

            fn checked_rem(self, v: $ty) -> Option<$ty> {
                let result = self % v;
                match !result.is_infinite() && !result.is_nan() && !result.is_subnormal() {
                    true => Some(result),
                    false => None
                }
            }

            fn checked_cos(self) -> Option<$ty> {
                let result = self.cos();
                match !result.is_infinite() && !result.is_nan() && !result.is_subnormal() {
                    true => Some(result),
                    false => None
                }
            }

            fn checked_sin(self) -> Option<$ty> {
                let result = self.sin();
                match !result.is_infinite() && !result.is_nan() && !result.is_subnormal() {
                    true => Some(result),
                    false => None
                }
            }

            fn checked_tan(self) -> Option<$ty> {
                let result = self.tan();
                match !result.is_infinite() && !result.is_nan() && !result.is_subnormal() {
                    true => Some(result),
                    false => None
                }
            }

            fn checked_acos(self) -> Option<$ty> {
                let result = self.acos();
                match !result.is_infinite() && !result.is_nan() && !result.is_subnormal() {
                    true => Some(result),
                    false => None
                }
            }

            fn checked_asin(self) -> Option<$ty> {
                let result = self.asin();
                match !result.is_infinite() && !result.is_nan() && !result.is_subnormal() {
                    true => Some(result),
                    false => None
                }
            }

            fn checked_atan(self) -> Option<$ty> {
                let result = self.atan();
                match !result.is_infinite() && !result.is_nan() && !result.is_subnormal() {
                    true => Some(result),
                    false => None
                }
            }

            fn checked_exp(self) -> Option<$ty> {
                let result = self.exp();
                match !result.is_infinite() && !result.is_nan() && !result.is_subnormal() {
                    true => Some(result),
                    false => None
                }
            }

            fn checked_ln(self) -> Option<$ty> {
                let result = self.ln();
                match !result.is_infinite() && !result.is_nan() && !result.is_subnormal() {
                    true => Some(result),
                    false => None
                }
            }

            fn checked_fract(self) -> Option<$ty> {
                let result = self.fract();
                match !result.is_infinite() && !result.is_nan() && !result.is_subnormal() {
                    true => Some(result),
                    false => None
                }
            }

            fn checked_trunc(self) -> Option<$ty> {
                let result = self.trunc();
                match !result.is_infinite() && !result.is_nan() && !result.is_subnormal() {
                    true => Some(result),
                    false => None
                }
            }

            fn checked_floor(self) -> Option<$ty> {
                let result = self.floor();
                match !result.is_infinite() && !result.is_nan() && !result.is_subnormal() {
                    true => Some(result),
                    false => None
                }
            }

            fn checked_ceil(self) -> Option<$ty> {
                let result = self.ceil();
                match !result.is_infinite() && !result.is_nan() && !result.is_subnormal() {
                    true => Some(result),
                    false => None
                }
            }

            fn checked_sqrt(self) -> Option<$ty> {
                let result = self.sqrt();
                match !result.is_infinite() && !result.is_nan() && !result.is_subnormal() {
                    true => Some(result),
                    false => None
                }
            }

            fn checked_sqr(self) -> Option<$ty> {
                let result = self * self;
                match !result.is_infinite() && !result.is_nan() && !result.is_subnormal() {
                    true => Some(result),
                    false => None
                }
            }
        }
    };
}

impl_checked_float!(f32);
impl_checked_float!(f64);

pub trait TryIntoCheck<T>: Sized {
    type Error;
    fn try_into(self) -> Result<T, Self::Error>;
}

impl TryIntoCheck<f32> for f64 {
    type Error = Stop;

    fn try_into(self) -> Result<f32, Self::Error> {
        let y = self as f32;
        match !y.is_infinite() && !y.is_nan() && !y.is_subnormal() {
            true => Ok(y),
            false => Err(error!(format!("Could not convert LReal {} to Real", self)))
        }
    }
}