// Copyright (c) the JPEG XL Project Authors. All rights reserved.
//
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

use std::{ops::Deref, sync::Arc};

use crate::{
    features::patches::PatchesDictionary,
    frame::ReferenceFrame,
    headers::extra_channels::ExtraChannelInfo,
    render::{RenderPipelineInPlaceStage, RenderPipelineStage},
};

#[allow(dead_code)]
pub struct PatchesStage {
    pub patches: PatchesDictionary,
    pub extra_channels: Vec<ExtraChannelInfo>,
    pub decoder_state: Arc<Vec<Option<ReferenceFrame>>>,
}

impl std::fmt::Display for PatchesStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "patches")
    }
}

impl RenderPipelineStage for PatchesStage {
    type Type = RenderPipelineInPlaceStage<f32>;

    fn uses_channel(&self, c: usize) -> bool {
        c < 3 + self.extra_channels.len()
    }

    fn process_row_chunk(
        &mut self,
        position: (usize, usize),
        xsize: usize,
        row: &mut [&mut [f32]],
    ) {
        self.patches.add_one_row(
            row,
            position,
            xsize,
            &self.extra_channels,
            self.decoder_state.deref(),
        );
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn process_row_chunk() {}
}
