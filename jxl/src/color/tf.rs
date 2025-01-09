// Copyright (c) the JPEG XL Project Authors. All rights reserved.
//
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use crate::util::eval_rational_poly;

const SRGB_POWTABLE_UPPER: [u8; 16] = [
    0x00, 0x0a, 0x19, 0x26, 0x32, 0x41, 0x4d, 0x5c, 0x68, 0x75, 0x83, 0x8f, 0xa0, 0xaa, 0xb9, 0xc6,
];

const SRGB_POWTABLE_LOWER: [u8; 16] = [
    0x00, 0xb7, 0x04, 0x0d, 0xcb, 0xe7, 0x41, 0x68, 0x51, 0xd1, 0xeb, 0xf2, 0x00, 0xb7, 0x04, 0x0d,
];

const PQ_M1: f64 = 2610.0 / 16384.0;
const PQ_M2: f64 = (2523.0 / 4096.0) * 128.0;
const PQ_C1: f64 = 3424.0 / 4096.0;
const PQ_C2: f64 = (2413.0 / 4096.0) * 32.0;
const PQ_C3: f64 = (2392.0 / 4096.0) * 32.0;

const PQ_EOTF_P: [f32; 5] = [
    2.6297566e-4,
    -6.235531e-3,
    7.386023e-1,
    2.6455317,
    5.500349e-1,
];
const PQ_EOTF_Q: [f32; 5] = [
    4.213501e2,
    -4.2873682e2,
    1.7436467e2,
    -3.3907887e1,
    2.6771877,
];

const PQ_INV_EOTF_P: [f32; 5] = [1.351392e-2, -1.095778, 5.522776e1, 1.492516e2, 4.838434e1];
const PQ_INV_EOTF_Q: [f32; 5] = [1.012416, 2.016708e1, 9.26371e1, 1.120607e2, 2.590418e1];
const PQ_INV_EOTF_P_SMALL: [f32; 5] = [
    9.863406e-6,
    3.881234e-1,
    1.352821e2,
    6.889862e4,
    -2.864824e5,
];
const PQ_INV_EOTF_Q_SMALL: [f32; 5] =
    [3.371868e1, 1.477719e3, 1.608477e4, -4.389884e4, -2.072546e5];

/// Converts the linear samples with the sRGB transfer curve.
// Fast linear to sRGB conversion, ported from libjxl. Max error ~1.7e-4
pub fn linear_to_srgb_fast(samples: &mut [f32]) {
    for s in samples {
        let v = s.to_bits() & 0x7fff_ffff;
        let v_adj = f32::from_bits((v | 0x3e80_0000) & 0x3eff_ffff);
        let pow = 0.059914046f32;
        let pow = pow * v_adj - 0.10889456;
        let pow = pow * v_adj + 0.107963754;
        let pow = pow * v_adj + 0.018092343;

        // `mul` won't be used when `v` is small.
        let idx = (v >> 23).wrapping_sub(118) as usize & 0xf;
        let mul = 0x4000_0000
            | (u32::from(SRGB_POWTABLE_UPPER[idx]) << 18)
            | (u32::from(SRGB_POWTABLE_LOWER[idx]) << 10);

        let v = f32::from_bits(v);
        let small = v * 12.92;
        let acc = pow * f32::from_bits(mul) - 0.055;

        *s = if v <= 0.0031308 { small } else { acc }.copysign(*s);
    }
}

/// Converts the linear samples with the sRGB transfer curve.
// Max error ~5e-7
pub fn linear_to_srgb(samples: &mut [f32]) {
    #[allow(clippy::excessive_precision)]
    const P: [f32; 5] = [
        -5.135152395e-4,
        5.287254571e-3,
        3.903842876e-1,
        1.474205315,
        7.352629620e-1,
    ];

    #[allow(clippy::excessive_precision)]
    const Q: [f32; 5] = [
        1.004519624e-2,
        3.036675394e-1,
        1.340816930,
        9.258482155e-1,
        2.424867759e-2,
    ];

    for x in samples {
        let a = x.abs();
        *x = if a <= 0.0031308 {
            a * 12.92
        } else {
            eval_rational_poly(a.sqrt(), P, Q)
        }
        .copysign(*x);
    }
}

/// Converts samples in sRGB transfer curve to linear. Inverse of `linear_to_srgb`.
pub fn srgb_to_linear(samples: &mut [f32]) {
    #[allow(clippy::excessive_precision)]
    const P: [f32; 5] = [
        2.200248328e-4,
        1.043637593e-2,
        1.624820318e-1,
        7.961564959e-1,
        8.210152774e-1,
    ];

    #[allow(clippy::excessive_precision)]
    const Q: [f32; 5] = [
        2.631846970e-1,
        1.076976492,
        4.987528350e-1,
        -5.512498495e-2,
        6.521209011e-3,
    ];

    for x in samples {
        let a = x.abs();
        *x = if a <= 0.04045 {
            a / 12.92
        } else {
            eval_rational_poly(a, P, Q)
        }
        .copysign(*x);
    }
}

/// Converts the linear samples with the BT.709 transfer curve.
pub fn linear_to_bt709(samples: &mut [f32]) {
    for s in samples {
        let a = s.abs();
        *s = if a <= 0.018 {
            a * 4.5
        } else {
            crate::util::fast_powf(a, 0.45).mul_add(1.099, -0.099)
        }
        .copysign(*s);
    }
}

/// Converts samples in BT.709 transfer curve to linear. Inverse of `linear_to_bt709`.
pub fn bt709_to_linear(samples: &mut [f32]) {
    for s in samples {
        let a = s.abs();
        *s = if a <= 0.081 {
            a / 4.5
        } else {
            crate::util::fast_powf(a.mul_add(1.0 / 1.099, 0.099 / 1.099), 1.0 / 0.45)
        }
        .copysign(*s);
    }
}

/// Converts linear sample to PQ signal using PQ inverse EOTF, where linear sample value of 1.0
/// represents `intensity_target` display nits.
///
/// This version uses original EOTF using double precision arithmetic internally.
pub fn linear_to_pq_precise(intensity_target: f32, samples: &mut [f32]) {
    let mult = intensity_target as f64 * 10000f64.recip();

    for s in samples {
        if *s == 0.0 {
            continue;
        }

        let a = s.abs() as f64;
        let xp = (a * mult).powf(PQ_M1);
        let num = PQ_C1 + xp * PQ_C2;
        let den = 1.0 + xp * PQ_C3;
        let e = (num / den).powf(PQ_M2);
        *s = (e as f32).copysign(*s);
    }
}

/// Converts PQ signal to linear sample using PQ EOTF, where linear sample value of 1.0 represents
/// `intensity_target` display nits.
///
/// This version uses original EOTF using double precision arithmetic internally.
pub fn pq_to_linear_precise(intensity_target: f32, samples: &mut [f32]) {
    let mult = 10000.0 / intensity_target as f64;

    for s in samples {
        if *s == 0.0 {
            continue;
        }

        let a = s.abs() as f64;
        let xp = a.powf(PQ_M2.recip());
        let num = (xp - PQ_C1).max(0.0);
        let den = PQ_C2 - PQ_C3 * xp;
        let y = (num / den).powf(PQ_M1.recip());
        *s = ((y * mult) as f32).copysign(*s);
    }
}

/// Converts linear sample to PQ signal using PQ inverse EOTF, where linear sample value of 1.0
/// represents `intensity_target` display nits.
///
/// This version uses approximate curve using rational polynomial.
// Max error: ~7e-7 at intensity_target = 10000
pub fn linear_to_pq(intensity_target: f32, samples: &mut [f32]) {
    let y_mult = intensity_target * 10000f32.recip();

    for s in samples {
        let a = s.abs();
        let a_scaled = a * y_mult;
        let a_1_4 = a_scaled.sqrt().sqrt();

        let y = if a < 1e-4 {
            eval_rational_poly(a_1_4, PQ_INV_EOTF_P_SMALL, PQ_INV_EOTF_Q_SMALL)
        } else {
            eval_rational_poly(a_1_4, PQ_INV_EOTF_P, PQ_INV_EOTF_Q)
        };

        *s = y.copysign(*s);
    }
}

/// Converts PQ signal to linear sample using PQ EOTF, where linear sample value of 1.0 represents
/// `intensity_target` display nits.
///
/// This version uses approximate curve using rational polynomial.
// Max error: ~3e-6 at intensity_target = 10000
pub fn pq_to_linear(intensity_target: f32, samples: &mut [f32]) {
    let y_mult = 10000.0 / intensity_target;

    for s in samples {
        let a = s.abs();
        // a + a * a
        let x = a.mul_add(a, a);
        let y = eval_rational_poly(x, PQ_EOTF_P, PQ_EOTF_Q);
        *s = (y * y_mult).copysign(*s);
    }
}

#[cfg(test)]
mod test {
    use test_log::test;

    use super::*;
    use crate::util::test::assert_all_almost_eq;

    fn arb_samples(
        u: &mut arbtest::arbitrary::Unstructured,
    ) -> arbtest::arbitrary::Result<Vec<f32>> {
        const DENOM: u32 = 1 << 24;

        let mut samples = Vec::new();

        // uniform distribution in [-1.0, 1.0]
        while !u.is_empty() {
            let a: u32 = u.int_in_range(0..=DENOM)?;
            let signed: bool = u.arbitrary()?;
            let x = a as f32 / DENOM as f32;
            samples.push(if signed { -x } else { x });
        }

        Ok(samples)
    }

    #[test]
    fn srgb_roundtrip_arb() {
        arbtest::arbtest(|u| {
            let samples: Vec<f32> = arb_samples(u)?;
            let mut output = samples.clone();

            linear_to_srgb(&mut output);
            srgb_to_linear(&mut output);
            assert_all_almost_eq!(&output, &samples, 2e-6);
            Ok(())
        });
    }

    #[test]
    fn bt709_roundtrip_arb() {
        arbtest::arbtest(|u| {
            let samples: Vec<f32> = arb_samples(u)?;
            let mut output = samples.clone();

            linear_to_bt709(&mut output);
            bt709_to_linear(&mut output);
            assert_all_almost_eq!(&output, &samples, 5e-6);
            Ok(())
        });
    }

    #[test]
    fn linear_to_srgb_fast_arb() {
        arbtest::arbtest(|u| {
            let mut samples: Vec<f32> = arb_samples(u)?;
            let mut fast = samples.clone();

            linear_to_srgb(&mut samples);
            linear_to_srgb_fast(&mut fast);
            assert_all_almost_eq!(&samples, &fast, 1.7e-4);
            Ok(())
        });
    }

    #[test]
    fn linear_to_pq_arb() {
        arbtest::arbtest(|u| {
            let intensity_target = u.int_in_range(9900..=10100)? as f32;
            let mut samples: Vec<f32> = arb_samples(u)?;
            let mut precise = samples.clone();

            linear_to_pq(intensity_target, &mut samples);
            linear_to_pq_precise(intensity_target, &mut precise);
            // Error seems to increase at intensity_target < 10000
            assert_all_almost_eq!(&samples, &precise, 8e-7);
            Ok(())
        });
    }

    #[test]
    fn pq_to_linear_arb() {
        arbtest::arbtest(|u| {
            let intensity_target = u.int_in_range(9900..=10100)? as f32;
            let mut samples: Vec<f32> = arb_samples(u)?;
            let mut precise = samples.clone();

            pq_to_linear(intensity_target, &mut samples);
            pq_to_linear_precise(intensity_target, &mut precise);
            assert_all_almost_eq!(&samples, &precise, 3e-6);
            Ok(())
        });
    }
}