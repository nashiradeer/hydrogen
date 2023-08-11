use serde::{Deserialize, Serialize};

/// Audio filters natively supported by Lavalink.
#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Filters {
    /// Lets you adjust the player volume from 0.0 to 5.0 where 1.0 is 100%. Values >1.0 may cause clipping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub volume: Option<f32>,

    /// Lets you adjust 15 different bands.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equalizer: Option<Vec<Equalizer>>,

    /// Lets you eliminate part of a band, usually targeting vocals.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub karaoke: Option<Karaoke>,

    /// Lets you change the speed, pitch, and rate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timescale: Option<Timescale>,

    /// Lets you create a shuddering effect, where the volume quickly oscillates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tremolo: Option<Tremolo>,

    /// Lets you create a shuddering effect, where the pitch quickly oscillates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vibrato: Option<Vibrato>,

    /// Lets you rotate the sound around the stereo channels/user headphones aka Audio Panning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation: Option<Rotation>,

    /// Lets you distort the audio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distortion: Option<Distortion>,

    /// Lets you mix both channels (left and right).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_mix: Option<ChannelMix>,

    /// Lets you filter higher frequencies.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub low_pass: Option<LowPass>,
}

/// There are 15 bands (0-14) that can be changed. `gain` is the multiplier for the given band. The default value is 0. Valid values range from -0.25 to 1.0, where -0.25 means the given band is completely muted, and 0.25 means it is doubled. Modifying the gain could also change the volume of the output.
#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Equalizer {
    /// The band (0 to 14).
    ///
    /// | Band | Frequency |
    /// | ---- | --------- |
    /// |  0   | 25 Hz     |
    /// |  1   | 40 Hz     |
    /// |  2   | 63 Hz     |
    /// |  3   | 100 Hz    |
    /// |  4   | 160 Hz    |
    /// |  5   | 250 Hz    |
    /// |  6   | 400 Hz    |
    /// |  7   | 630 Hz    |
    /// |  8   | 1000 Hz   |
    /// |  9   | 1600 Hz   |
    /// |  10  | 2500 Hz   |
    /// |  11  | 4000 Hz   |
    /// |  12  | 6300 Hz   |
    /// |  13  | 10000 Hz  |
    /// |  14  | 16000 Hz  |
    pub band: u8,

    /// The gain (-0.25 to 1.0).
    pub gain: f32,
}

/// Uses equalization to eliminate part of a band, usually targeting vocals.
#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Karaoke {
    /// The level (0 to 1.0 where 0.0 is no effect and 1.0 is full effect).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<f32>,

    /// The mono level (0 to 1.0 where 0.0 is no effect and 1.0 is full effect).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mono_level: Option<f32>,

    /// The filter band.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_band: Option<f32>,

    /// The filter width.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_width: Option<f32>,
}

/// Changes the speed, pitch, and rate. All default to 1.0.
#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Timescale {
    /// The playback speed 0.0 ≤ x.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f32>,

    /// The pitch 0.0 ≤ x.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pitch: Option<f32>,

    /// The rate 0.0 ≤ x.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate: Option<f32>,
}

/// Uses amplification to create a shuddering effect, where the volume quickly oscillates.
///
/// https://en.wikipedia.org/wiki/File:Fuse_Electronics_Tremolo_MK-III_Quick_Demo.ogv
#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Tremolo {
    /// The frequency 0.0 < x.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency: Option<f32>,

    /// The tremolo depth 0.0 < x ≤ 1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<f32>,
}

// Similar to `Tremolo`. While `Tremolo` oscillates the volume, `Vibrato` oscillates the pitch.
#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Vibrato {
    /// The frequency 0.0 < x ≤ 14.0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency: Option<f32>,

    /// The vibrato depth 0.0 < x ≤ 1.0.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depth: Option<f32>,
}

/// Rotates the sound around the stereo channels/user headphones aka Audio Panning. It can produce an effect similar to https://youtu.be/QB9EB8mTKcc (without the reverb).
#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Rotation {
    /// The frequency of the audio rotating around the listener in Hz. 0.2 is similar to the example video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rotation_hz: Option<f32>,
}

/// Distortion effect. It can generate some pretty unique audio effects.
#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Distortion {
    /// The sin offset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sin_offset: Option<f32>,

    /// The sin scale.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sin_scale: Option<f32>,

    /// The cos offset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cos_offset: Option<f32>,

    /// The cos scale.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cos_scale: Option<f32>,

    /// The tan offset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tan_offset: Option<f32>,

    /// The tan scale.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tan_scale: Option<f32>,

    /// The offset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<f32>,

    /// The scale.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f32>,
}

/// Mixes both channels (left and right), with a configurable factor on how much each channel affects the other. With the defaults, both channels are kept independent of each other. Setting all factors to 0.5 means both channels get the same audio.
#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelMix {
    /// The left to left channel mix factor. (0.0 ≤ x ≤ 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left_to_left: Option<f32>,

    /// The left to right channel mix factor. (0.0 ≤ x ≤ 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left_to_right: Option<f32>,

    /// The right to left channel mix factor. (0.0 ≤ x ≤ 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right_to_left: Option<f32>,

    /// The right to right channel mix factor. (0.0 ≤ x ≤ 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right_to_right: Option<f32>,
}

/// Higher frequencies get suppressed, while lower frequencies pass through this filter, thus the name low pass. Any smoothing values equal to, or less than 1.0 will disable the filter.
#[derive(Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LowPass {
    /// The smoothing factor. (1.0 < x)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smoothing: Option<f32>,
}
