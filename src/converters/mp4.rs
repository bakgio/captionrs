use std::collections::VecDeque;
use std::convert::TryInto;
use std::io::{Cursor, Read, Seek};

use mp4forge::FourCc;
use mp4forge::boxes::iso14496_12::{
    Mdhd, TFHD_DEFAULT_SAMPLE_DURATION_PRESENT, TRUN_SAMPLE_DURATION_PRESENT, Tfdt, Tfhd, Trun,
};
use mp4forge::boxes::iso14496_30::{CuePayloadBox, CueSettingsBox, WebVTTConfigurationBox};
use mp4forge::codec::{CodecBox, ImmutableBox};
use mp4forge::walk::{WalkControl, WalkError, WalkHandle, walk_structure};

use crate::converters::{BaseConverter, SMPTEConverter, WebVTTConverter};
use crate::subripfile::{SubRipFile, SubtitleError};
use crate::utils::time::timestamp_from_ms;

#[cfg(feature = "async")]
use crate::converters::base::AsyncBaseConverter;
#[cfg(feature = "async")]
use tokio::io::{AsyncRead, AsyncReadExt};

const DEFAULT_TIMESCALE: u32 = 1000;
const FTYP_HEADER: &[u8] = b"\x00\x00\x00\x1cftyp";
const MDAT: FourCc = FourCc::from_bytes(*b"mdat");
const MDHD: FourCc = FourCc::from_bytes(*b"mdhd");
const MINF: FourCc = FourCc::from_bytes(*b"minf");
const MDIA: FourCc = FourCc::from_bytes(*b"mdia");
const MOOF: FourCc = FourCc::from_bytes(*b"moof");
const MOOV: FourCc = FourCc::from_bytes(*b"moov");
const PAYL: FourCc = FourCc::from_bytes(*b"payl");
const STBL: FourCc = FourCc::from_bytes(*b"stbl");
const STSD: FourCc = FourCc::from_bytes(*b"stsd");
const STTG: FourCc = FourCc::from_bytes(*b"sttg");
const TFDT: FourCc = FourCc::from_bytes(*b"tfdt");
const TFHD: FourCc = FourCc::from_bytes(*b"tfhd");
const TRAF: FourCc = FourCc::from_bytes(*b"traf");
const TRAK: FourCc = FourCc::from_bytes(*b"trak");
const TRUN: FourCc = FourCc::from_bytes(*b"trun");
const STYP_HEADER: &[u8] = b"\x00\x00\x00\x18styp";
const VTTC: FourCc = FourCc::from_bytes(*b"vttc");
const VTTE: FourCc = FourCc::from_bytes(*b"vtte");
const VTT_CONFIGURATION: FourCc = FourCc::from_bytes(*b"vttC");
const WVTT: FourCc = FourCc::from_bytes(*b"wvtt");

#[derive(Clone)]
pub struct ISMTConverter;

impl ISMTConverter {
    pub fn new() -> Self {
        Self
    }

    fn parse_bytes(&self, data: &[u8]) -> Result<SubRipFile, SubtitleError> {
        if !data.is_empty() && !appears_to_be_mp4(data) {
            return Ok(SubRipFile::new(None));
        }

        let mut srt = SubRipFile::new(None);
        let mut conversion_error = None;

        for segment in split_segmented_mp4(data) {
            let mut stream = Cursor::new(segment);
            walk_structure(&mut stream, |handle| {
                if conversion_error.is_some() {
                    return Ok(WalkControl::Continue);
                }

                if handle.path().len() != 1 || handle.info().box_type() != MDAT {
                    return Ok(WalkControl::Continue);
                }

                let payload = read_box_payload(handle)?;
                match SMPTEConverter::new().from_bytes(&payload) {
                    Ok(new_srt) => append_fragment(&mut srt, new_srt),
                    Err(error) => conversion_error = Some(error),
                }

                Ok(WalkControl::Continue)
            })
            .map_err(map_mp4_error)?;
        }

        if let Some(error) = conversion_error {
            return Err(error);
        }

        Ok(srt)
    }
}

impl Default for ISMTConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseConverter for ISMTConverter {
    /// ISMT (DFXP in MP4) subtitle converter.
    fn parse<R: Read>(&self, mut stream: R) -> Result<SubRipFile, SubtitleError> {
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer)?;
        self.parse_bytes(&buffer)
    }
}

#[derive(Clone)]
pub struct WVTTConverter;

impl WVTTConverter {
    pub fn new() -> Self {
        Self
    }

    fn parse_bytes(&self, data: &[u8]) -> Result<SubRipFile, SubtitleError> {
        if !data.is_empty() && !appears_to_be_mp4(data) {
            return Ok(SubRipFile::new(None));
        }

        let mut sample_durations = VecDeque::new();
        let mut vtt_lines = Vec::new();
        let mut timescale = DEFAULT_TIMESCALE;
        let mut base_decode_time = 0u64;
        let mut default_sample_duration = None;
        let mut conversion_error = None;

        for segment in split_segmented_mp4(data) {
            let mut stream = Cursor::new(segment);
            walk_structure(&mut stream, |handle| {
                if conversion_error.is_some() {
                    return Ok(WalkControl::Continue);
                }

                let box_type = handle.info().box_type();
                if handle.path().len() == 1 {
                    if box_type == MOOF {
                        base_decode_time = 0;
                        default_sample_duration = None;
                        return Ok(WalkControl::Descend);
                    }

                    if box_type == MDAT {
                        let payload = read_box_payload(handle)?;
                        if let Err(error) =
                            self.parse_vtt_payload(&payload, &mut sample_durations, &mut vtt_lines)
                        {
                            conversion_error = Some(error);
                        }
                    }

                    return Ok(WalkControl::Continue);
                }

                match box_type {
                    MOOV | TRAK | MDIA | MINF | STBL | STSD | TRAF | WVTT => {
                        return Ok(WalkControl::Descend);
                    }
                    MDHD if timescale == DEFAULT_TIMESCALE => {
                        match read_typed_box::<_, Mdhd>(handle) {
                            Ok(mdhd) => timescale = mdhd.timescale.max(1),
                            Err(error) => conversion_error = Some(error),
                        }
                    }
                    VTT_CONFIGURATION if vtt_lines.is_empty() => {
                        match read_typed_box::<_, WebVTTConfigurationBox>(handle) {
                            Ok(header) => vtt_lines.push(format!("{}\n\n", header.config)),
                            Err(error) => conversion_error = Some(error),
                        }
                    }
                    TFDT => match read_typed_box::<_, Tfdt>(handle) {
                        Ok(tfdt) => base_decode_time = tfdt.base_media_decode_time(),
                        Err(error) => conversion_error = Some(error),
                    },
                    TFHD => match read_typed_box::<_, Tfhd>(handle) {
                        Ok(tfhd) => {
                            default_sample_duration =
                                (tfhd.flags() & TFHD_DEFAULT_SAMPLE_DURATION_PRESENT != 0)
                                    .then_some(tfhd.default_sample_duration);
                        }
                        Err(error) => conversion_error = Some(error),
                    },
                    TRUN => match read_typed_box::<_, Trun>(handle) {
                        Ok(trun) => self.process_fragment_timing(
                            base_decode_time,
                            &trun,
                            default_sample_duration,
                            timescale,
                            &mut sample_durations,
                        ),
                        Err(error) => conversion_error = Some(error),
                    },
                    _ => {}
                }

                Ok(WalkControl::Continue)
            })
            .map_err(map_mp4_error)?;
        }

        if let Some(error) = conversion_error {
            return Err(error);
        }

        if !vtt_lines.is_empty() && !vtt_lines[0].starts_with("WEBVTT") {
            vtt_lines.insert(0, "WEBVTT\n\n".to_string());
        }

        WebVTTConverter::new().from_string(&vtt_lines.join(""))
    }

    fn process_fragment_timing(
        &self,
        base_decode_time: u64,
        trun: &Trun,
        default_sample_duration: Option<u32>,
        timescale: u32,
        sample_durations: &mut VecDeque<SampleDuration>,
    ) {
        let timescale = timescale.max(1) as f64;
        let mut start_offset = base_decode_time as i64;
        let mut duration = 0i64;

        for (index, sample) in trun.entries.iter().enumerate() {
            let sample_duration = if trun.flags() & TRUN_SAMPLE_DURATION_PRESENT != 0 {
                sample.sample_duration
            } else {
                default_sample_duration.unwrap_or_default()
            };

            // Keep the accumulated run timing semantics that the existing WVTT behavior expects.
            start_offset += trun.sample_composition_time_offset(index);
            duration += i64::from(sample_duration);

            sample_durations.push_back(SampleDuration {
                start_ms: (start_offset as f64 / timescale) * 1000.0,
                end_ms: ((start_offset + duration) as f64 / timescale) * 1000.0,
            });
        }
    }

    fn parse_vtt_payload(
        &self,
        payload: &[u8],
        sample_durations: &mut VecDeque<SampleDuration>,
        vtt_lines: &mut Vec<String>,
    ) -> Result<(), SubtitleError> {
        let mut new_start = None;
        let mut previous_end = None;
        let mut conversion_error = None;
        let mut stream = Cursor::new(payload);

        walk_structure(&mut stream, |handle| {
            if conversion_error.is_some() {
                return Ok(WalkControl::Continue);
            }

            if handle.path().len() != 1 {
                return Ok(WalkControl::Continue);
            }

            let Some(sample_duration) = sample_durations.pop_front() else {
                return Ok(WalkControl::Continue);
            };

            let box_type = handle.info().box_type();
            let payload = read_box_payload(handle)?;
            let (settings, cue_text) = match box_type {
                VTTC => match parse_vtt_cue_payload(&payload) {
                    Ok(cue) => cue,
                    Err(error) => {
                        conversion_error = Some(error);
                        return Ok(WalkControl::Continue);
                    }
                },
                PAYL => (None, Some(String::from_utf8_lossy(&payload).into_owned())),
                _ => (None, None),
            };

            let mut start_ms = if box_type == VTTC {
                previous_end.unwrap_or(sample_duration.end_ms)
            } else {
                sample_duration.start_ms
            };
            let end_ms = sample_duration.end_ms;
            previous_end = Some(end_ms);

            // Empty cues move the next cue start forward without producing subtitle text.
            if box_type == VTTE {
                new_start = Some(end_ms);
                return Ok(WalkControl::Continue);
            }

            if let Some(forced_start) = new_start.take() {
                start_ms = forced_start;
            }

            if let Some(cue_text) = cue_text {
                vtt_lines.push(format!(
                    "{} --> {} {}\n{}\n\n",
                    timestamp_from_ms(start_ms),
                    timestamp_from_ms(end_ms),
                    settings.unwrap_or_else(|| "None".to_string()),
                    cue_text
                ));
            }

            Ok(WalkControl::Continue)
        })
        .map_err(map_mp4_error)?;

        if let Some(error) = conversion_error {
            return Err(error);
        }

        Ok(())
    }
}

impl Default for WVTTConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl BaseConverter for WVTTConverter {
    /// WVTT (WebVTT in MP4) subtitle converter.
    fn parse<R: Read>(&self, mut stream: R) -> Result<SubRipFile, SubtitleError> {
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer)?;
        self.parse_bytes(&buffer)
    }
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl AsyncBaseConverter for ISMTConverter {
    /// Async ISMT subtitle converter for library integrations.
    async fn parse_async<R: AsyncRead + Unpin + Send>(
        &self,
        mut stream: R,
    ) -> Result<SubRipFile, SubtitleError> {
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer).await?;
        let converter = self.clone();
        crate::async_utils::run_blocking(move || converter.parse_bytes(&buffer)).await
    }
}

#[cfg(feature = "async")]
#[async_trait::async_trait]
impl AsyncBaseConverter for WVTTConverter {
    /// Async WVTT subtitle converter for library integrations.
    async fn parse_async<R: AsyncRead + Unpin + Send>(
        &self,
        mut stream: R,
    ) -> Result<SubRipFile, SubtitleError> {
        let mut buffer = Vec::new();
        stream.read_to_end(&mut buffer).await?;
        let converter = self.clone();
        crate::async_utils::run_blocking(move || converter.parse_bytes(&buffer)).await
    }
}

#[derive(Debug, Clone, Copy)]
struct SampleDuration {
    start_ms: f64,
    end_ms: f64,
}

fn append_fragment(srt: &mut SubRipFile, new_srt: SubRipFile) {
    if !srt.is_empty() && !new_srt.is_empty() {
        let last_line = srt.get(srt.len() - 1);
        let first_line = new_srt.get(0);

        if let (Some(last_line), Some(first_line)) = (last_line, first_line)
            && last_line.start > first_line.start
        {
            let mut shifted_srt = new_srt;
            shifted_srt.offset(last_line.end);
            srt.extend(shifted_srt);
            return;
        }
    }

    srt.extend(new_srt);
}

fn split_segmented_mp4(data: &[u8]) -> Vec<&[u8]> {
    let Some(_) = find_subslice(data, STYP_HEADER) else {
        return vec![data];
    };

    let mut segments = Vec::new();
    let mut position = find_subslice(data, FTYP_HEADER).unwrap_or(0);
    let mut previous_position = position;

    while let Some(found_position) = find_subslice_from(data, STYP_HEADER, position) {
        let segment = &data[previous_position..found_position];
        segments.push(segment);
        previous_position = found_position;
        position = found_position + segment.len();
    }

    segments
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    find_subslice_from(haystack, needle, 0)
}

fn find_subslice_from(haystack: &[u8], needle: &[u8], start: usize) -> Option<usize> {
    haystack
        .get(start..)?
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|offset| start + offset)
}

fn appears_to_be_mp4(data: &[u8]) -> bool {
    if data.len() < 8 {
        return false;
    }

    if find_subslice(data, FTYP_HEADER).is_some() || find_subslice(data, STYP_HEADER).is_some() {
        return true;
    }

    let declared_size = u32::from_be_bytes(
        data[..4]
            .try_into()
            .expect("top-level MP4 size header requires exactly 4 bytes"),
    ) as usize;
    let box_type = &data[4..8];

    is_ascii_box_type(box_type)
        && (declared_size == 0 || declared_size == 1 || declared_size <= data.len())
}

fn is_ascii_box_type(box_type: &[u8]) -> bool {
    box_type
        .iter()
        .all(|byte| byte.is_ascii_alphanumeric() || *byte == b' ')
}

fn parse_vtt_cue_payload(
    payload: &[u8],
) -> Result<(Option<String>, Option<String>), SubtitleError> {
    let mut settings = None;
    let mut cue_text = None;
    let mut conversion_error = None;
    let mut stream = Cursor::new(payload);

    walk_structure(&mut stream, |handle| {
        if conversion_error.is_some() {
            return Ok(WalkControl::Continue);
        }

        if handle.path().len() != 1 {
            return Ok(WalkControl::Continue);
        }

        match handle.info().box_type() {
            STTG => match read_typed_box::<_, CueSettingsBox>(handle) {
                Ok(cue_settings) => settings = Some(cue_settings.settings),
                Err(error) => conversion_error = Some(error),
            },
            PAYL => match read_typed_box::<_, CuePayloadBox>(handle) {
                Ok(payload_box) => cue_text = Some(payload_box.cue_text),
                Err(error) => conversion_error = Some(error),
            },
            _ => {}
        }

        Ok(WalkControl::Continue)
    })
    .map_err(map_mp4_error)?;

    if let Some(error) = conversion_error {
        return Err(error);
    }

    Ok((settings, cue_text))
}

fn read_box_payload<R: Read + Seek>(handle: &mut WalkHandle<'_, R>) -> Result<Vec<u8>, WalkError> {
    let mut payload = Vec::new();
    handle.read_data(&mut payload)?;
    Ok(payload)
}

fn read_typed_box<R, T>(handle: &mut WalkHandle<'_, R>) -> Result<T, SubtitleError>
where
    R: Read + Seek,
    T: CodecBox + Clone + 'static,
{
    let box_type = handle.info().box_type();
    let (payload, _) = handle.read_payload().map_err(map_mp4_error)?;
    payload
        .as_ref()
        .as_any()
        .downcast_ref::<T>()
        .cloned()
        .ok_or_else(|| SubtitleError::Parse(format!("Unexpected MP4 payload type for {box_type}")))
}

fn map_mp4_error(error: impl std::fmt::Display) -> SubtitleError {
    SubtitleError::Parse(format!("MP4 parse error: {error}"))
}
