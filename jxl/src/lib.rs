// Copyright (c) the JPEG XL Project Authors. All rights reserved.
//
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

#![deny(unsafe_code)]
pub mod bit_reader;
pub mod color;
pub mod container;
pub mod entropy_coding;
pub mod error;
pub mod features;
pub mod frame;
pub mod headers;
pub mod icc;
pub mod image;
pub mod render;
pub mod util;

// TODO: Move these to a more appropriate location.
const BLOCK_DIM: usize = 8;
const SIGMA_PADDING: usize = 2;
#[allow(clippy::excessive_precision)]
const MIN_SIGMA: f32 = -3.90524291751269967465540850526868;
