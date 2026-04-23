// Common converter test utilities and shared test data

mod async_api;
mod equivalence;
mod format_specific;
mod sync_api;

pub const WEBVTT_SAMPLE: &str = r#"WEBVTT

1
00:00:01.000 --> 00:00:03.000
Hello, world!

2
00:00:04.000 --> 00:00:06.000
<i>This is italicized text.</i>

3
00:00:07.000 --> 00:00:09.000
Multiple lines
on this subtitle."#;

pub const WEBVTT_SAMPLE_SRT: &str = concat!(
    "1\n",
    "00:00:01,000 --> 00:00:03,000\n",
    "Hello, world!\n\n",
    "2\n",
    "00:00:04,000 --> 00:00:06,000\n",
    "<i>This is italicized text.</i>\n\n",
    "3\n",
    "00:00:07,000 --> 00:00:09,000\n",
    "Multiple lines\n",
    "on this subtitle.\n\n"
);

pub const FILE_API_WEBVTT_SAMPLE: &str = r#"WEBVTT

00:00:01.000 --> 00:00:02.000
Hello World

00:00:03.000 --> 00:00:04.000
<i>This is italic text</i>

00:00:05.000 --> 00:00:06.000 position:50% align:middle
Centered subtitle

00:00:07.000 --> 00:00:08.000
<v Speaker>Speaker name example</v>

00:00:09.000 --> 00:00:10.000
[background music playing]

00:00:11.000 --> 00:00:12.000
Line with <ruby>Ruby<rt>annotation</rt></ruby> text"#;

pub const SMPTE_SAMPLE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<tt xmlns="http://www.w3.org/ns/ttml" 
    xmlns:tts="http://www.w3.org/ns/ttml#styling"
    xml:lang="en">
  <head>
    <styling>
      <style xml:id="italic" tts:fontStyle="italic"/>
    </styling>
  </head>
  <body>
    <div>
      <p begin="00:00:01.000" end="00:00:03.000">Hello, world!</p>
      <p begin="00:00:04.000" end="00:00:06.000" style="italic">This is italicized text.</p>
      <p begin="00:00:07.000" end="00:00:09.000">Multiple lines<br/>on this subtitle.</p>
    </div>
  </body>
</tt>"#;

pub const WEBVTT_RUBY_SAMPLE: &str = r#"
00:03:14.945 --> 00:03:16.238 line:95%,end
第二<ruby>九龍<rt>クーロン</rt></ruby>って 決して
住みやすい場所じゃないと思うけど
"#;

pub const TTML_RUBY_SAMPLE: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<tt xmlns="http://www.w3.org/ns/ttml" xmlns:tts="http://www.w3.org/ns/ttml#styling" xmlns:xml="http://www.w3.org/XML/1998/namespace" xml:lang="ja">
<head>
<styling>
<style xml:id="ruby" tts:ruby="text" tts:rubyAlign="center" tts:rubyPosition="outside"/>
</styling>
<layout>
</layout>
</head>
<body>
<div>
<p xml:id="s" begin="00:00:07.967" end="00:00:09.385">（風子<span style="ruby">ふうこ</span>）あっ…</p>
</div>
</body>
</tt>"#;

pub const SPEAKER_TAG_TEST: &[u8] = b"1
00:00:01.000 --> 00:00:03.000
- <v ID>TESTY TESTERSON:</v>
<v Testerson>This is a test, if my name isn't Testy Testerson!</v>
";

pub const NESTED_ITALICS_TEST: &[u8] = b"1
00:00:01.000 --> 00:00:03.000
<c.font-family_monospace><c.background-color_000000.font-style_italic>He'll open up your heart</c></c>
";

pub const WEBVTT_STYLE_BLOCK_TEST: &str = r#"WEBVTT

STYLE
::cue(.narration) {
  font-style: italic;
}

00:00:01.000 --> 00:00:02.000
<c.narration>Styled text</c>
"#;

pub const WEBVTT_STYLE_BLOCK_INLINE_RULE_TEST: &str = r#"WEBVTT

STYLE
::cue(.narration) { color: lime; font-style: italic; }

00:00:01.000 --> 00:00:02.000
<c.narration>Styled text</c>
"#;

pub const WEBVTT_STYLE_BLOCK_SRT: &str = concat!(
    "1\n",
    "00:00:01,000 --> 00:00:02,000\n",
    "<i>Styled text</i>\n\n"
);

pub const POSITION_SORT_TEST: &[u8] = b"1
00:00:42.417 --> 00:00:44.503 position:25.24%,start align:start size:61.43% line:84.62%
that can happen in sports?\"

2
00:00:42.417 --> 00:00:44.503 position:29.05%,start align:start size:53.81% line:79.29%
\"What is the worst thing

3
00:00:52.417 --> 00:00:54.503 position:25.24%,start align:start size:61.43% line:84.62%
that can happen in sports?\"

4
00:00:52.430 --> 00:00:54.603 position:29.05%,start align:start size:53.81% line:79.29%
\"What is the worst thing

5
00:00:58.417 --> 00:00:58.503 position:25.24%,start align:start size:61.43% line:84.62%
First line.

6
00:00:58.417 --> 00:00:58.503 position:25.24%,start align:start size:61.43% line:84.62%
Second line.

7
00:00:58.417 --> 00:00:58.503 position:25.24%,start align:start size:61.43% line:84.62%
Third line.
";

pub const BILIBILI_SAMPLE: &str = r#"{"body":[
    {"from":1.0,"to":3.0,"location":2,"content":"Hello, world!"},
    {"from":4.0,"to":6.0,"location":2,"content":"This is a test subtitle."},
    {"from":7.0,"to":9.0,"location":2,"content":"Another test line."}
]}"#;

pub const BILIBILI_SAMPLE_SRT: &str = concat!(
    "1\n",
    "00:00:01,000 --> 00:00:03,000\n",
    "Hello, world!\n\n",
    "2\n",
    "00:00:04,000 --> 00:00:06,000\n",
    "This is a test subtitle.\n\n",
    "3\n",
    "00:00:07,000 --> 00:00:09,000\n",
    "Another test line.\n\n"
);

pub const BILIBILI_ALIGNMENT_TEST: &str =
    r#"{"body":[{"from":1.0,"to":3.0,"location":8,"content":"Top aligned"}]}"#;

pub const SAMI_SAMPLE: &str = r#"<SAMI>
<HEAD>
<TITLE>Sample SAMI file</TITLE>
</HEAD>
<BODY>
<SYNC Start="1000">
<P Class="CC">Hello, world!</P>
<SYNC Start="4000">
<P Class="CC">This is a test subtitle.</P>
<SYNC Start="7000">
<P Class="CC">Multiple lines<br>on this subtitle.</P>
</BODY>
</SAMI>"#;

pub const SAMI_SAMPLE_SRT: &str = concat!(
    "1\n",
    "00:00:01,000 --> 00:00:05,000\n",
    "Hello, world!\n\n",
    "2\n",
    "00:00:04,000 --> 00:00:08,000\n",
    "This is a test subtitle.\n\n",
    "3\n",
    "00:00:07,000 --> 00:00:11,000\n",
    "Multiple lines\n",
    "on this subtitle.\n\n"
);

pub const SMPTE_SAMPLE_SRT: &str = WEBVTT_SAMPLE_SRT;

pub const SMPTE_MULTIPLE_DOCUMENTS_SAMPLE: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
    "<tt xmlns=\"http://www.w3.org/ns/ttml\">\n",
    "  <body>\n",
    "    <div>\n",
    "      <p begin=\"00:00:01.000\" end=\"00:00:02.000\">First document cue</p>\n",
    "    </div>\n",
    "  </body>\n",
    "</tt>\n",
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
    "<tt xmlns=\"http://www.w3.org/ns/ttml\">\n",
    "  <body>\n",
    "    <div>\n",
    "      <p begin=\"00:00:02.500\" end=\"00:00:03.500\">Second document cue</p>\n",
    "    </div>\n",
    "  </body>\n",
    "</tt>\n"
);

pub const SMPTE_MULTIPLE_DOCUMENTS_SAMPLE_SRT: &str = concat!(
    "1\n",
    "00:00:01,000 --> 00:00:02,000\n",
    "First document cue\n\n",
    "2\n",
    "00:00:02,500 --> 00:00:03,500\n",
    "Second document cue\n\n"
);

pub const SMPTE_TICK_TIMING_SAMPLE: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
    "<tt xmlns=\"http://www.w3.org/ns/ttml\" ",
    "xmlns:ttp=\"http://www.w3.org/ns/ttml#parameter\" ttp:tickRate=\"10\">\n",
    "  <body>\n",
    "    <div>\n",
    "      <p begin=\"5t\" end=\"15t\">Tick based cue</p>\n",
    "    </div>\n",
    "  </body>\n",
    "</tt>\n"
);

pub const SMPTE_TICK_TIMING_SAMPLE_SRT: &str = concat!(
    "1\n",
    "00:00:00,500 --> 00:00:01,500\n",
    "Tick based cue\n\n"
);

pub const SMPTE_FRAME_TIMING_SAMPLE: &str = concat!(
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
    "<tt xmlns=\"http://www.w3.org/ns/ttml\" ",
    "xmlns:ttp=\"http://www.w3.org/ns/ttml#parameter\" ttp:frameRate=\"24\">\n",
    "  <body>\n",
    "    <div>\n",
    "      <p begin=\"00:00:01:12\" end=\"00:00:02:00\">Frame based cue</p>\n",
    "    </div>\n",
    "  </body>\n",
    "</tt>\n"
);

pub const SMPTE_FRAME_TIMING_SAMPLE_SRT: &str = concat!(
    "1\n",
    "00:00:01,500 --> 00:00:02,000\n",
    "Frame based cue\n\n"
);

pub const WEBVTT_EQUIVALENCE: &str = "WEBVTT\n\n00:00:01.000 --> 00:00:03.000\nTest subtitle\n\n00:00:04.000 --> 00:00:06.000\nAnother subtitle with  extra  spaces\n\n00:00:07.000 --> 00:00:09.000\n£ Musical  notes  £\n";

pub const SAMI_EQUIVALENCE: &str = "<SAMI><BODY><SYNC Start=1000><P>Test subtitle<SYNC Start=4000><P>Another subtitle with  extra  spaces<SYNC Start=7000><P>£ Musical  notes  £</BODY></SAMI>";

pub const BILIBILI_EQUIVALENCE: &str = r#"{"body":[{"from":1.0,"to":3.0,"location":2,"content":"Test subtitle"},{"from":4.0,"to":6.0,"location":2,"content":"Another subtitle with  extra  spaces"},{"from":7.0,"to":9.0,"location":2,"content":"£ Musical  notes  £"}]}"#;

pub const SMPTE_EQUIVALENCE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<tt xmlns="http://www.w3.org/ns/ttml" xmlns:tts="http://www.w3.org/ns/ttml#styling">
  <body>
    <div>
      <p begin="00:00:01.000" end="00:00:03.000">Test subtitle</p>
      <p begin="00:00:04.000" end="00:00:06.000">Another subtitle with  extra  spaces</p>  
      <p begin="00:00:07.000" end="00:00:09.000">£ Musical  notes  £</p>
    </div>
  </body>
</tt>"#;

// API method testing constants
pub const BILIBILI_API_TEST: &str = r#"{"body":[{"from":1.0,"to":3.0,"location":2,"content":"Test subtitle"},{"from":4.0,"to":6.0,"location":2,"content":"Another subtitle"}]}"#;
pub const BILIBILI_EMPTY: &str = r#"{"body":[]}"#;
pub const SAMI_API_TEST: &str = "<SAMI><BODY><SYNC Start=1000><P>Test subtitle<SYNC Start=4000><P>Another subtitle</BODY></SAMI>";
pub const WEBVTT_API_TEST: &str = "WEBVTT\n\n00:00:01.000 --> 00:00:03.000\nTest subtitle\n\n00:00:04.000 --> 00:00:06.000\nAnother subtitle\n";

pub const SEGMENTED_ISMT_SAMPLE_SRT: &str =
    concat!("1\n", "00:00:01,000 --> 00:00:02,000\n", "First cue\n\n");

pub const SEGMENTED_WVTT_SAMPLE_SRT: &str = "";

pub const SINGLE_FRAGMENT_WVTT_SAMPLE_SRT: &str = concat!(
    "1\n",
    "00:00:01,500 --> 00:00:01,900\n",
    "Single fragment plain cue\n\n"
);

pub fn build_segmented_ismt_sample() -> Vec<u8> {
    let first_ttml = r#"<?xml version="1.0" encoding="UTF-8"?>
<tt xmlns="http://www.w3.org/ns/ttml">
  <body>
    <div>
      <p begin="00:00:01.000" end="00:00:02.000">First cue</p>
    </div>
  </body>
</tt>"#;
    let second_ttml = r#"<?xml version="1.0" encoding="UTF-8"?>
<tt xmlns="http://www.w3.org/ns/ttml">
  <body>
    <div>
      <p begin="00:00:00.000" end="00:00:01.000">Second cue</p>
    </div>
  </body>
</tt>"#;

    [
        ftyp_box(),
        make_box(b"moov", &[]),
        styp_box(),
        make_box(b"mdat", first_ttml.as_bytes()),
        styp_box(),
        make_box(b"mdat", second_ttml.as_bytes()),
    ]
    .concat()
}

pub fn build_segmented_wvtt_sample() -> Vec<u8> {
    let moov_payload = [
        make_box(b"mdhd", &mdhd_payload(1000)),
        make_box(b"stsd", &stsd_payload("WEBVTT - Header")),
    ]
    .concat();
    let moof_payload = [
        make_box(b"tfdt", &tfdt_payload(1000)),
        make_box(b"trun", &trun_payload(&[(500, 200), (400, 100), (300, 0)])),
    ]
    .concat();
    let mdat_payload = [
        make_box(b"vtte", &[]),
        make_box(
            b"vttc",
            &[make_box(b"sttg", b"line:0%"), make_box(b"payl", b"Top cue")].concat(),
        ),
        make_box(b"vttc", &make_box(b"payl", b"Plain cue")),
    ]
    .concat();

    [
        ftyp_box(),
        make_box(b"moov", &moov_payload),
        styp_box(),
        make_box(b"moof", &moof_payload),
        make_box(b"mdat", &mdat_payload),
    ]
    .concat()
}

pub fn build_single_fragment_wvtt_sample() -> Vec<u8> {
    let moov_payload = [
        make_box(b"mdhd", &mdhd_payload(1000)),
        make_box(b"stsd", &stsd_payload("WEBVTT - Header")),
    ]
    .concat();
    let moof_payload = [
        make_box(b"tfdt", &tfdt_payload(1000)),
        make_box(b"trun", &trun_payload(&[(500, 0), (400, 0)])),
    ]
    .concat();
    let mdat_payload = [
        make_box(
            b"vttc",
            &[
                make_box(b"sttg", b"line:0%"),
                make_box(b"payl", b"Single fragment top cue"),
            ]
            .concat(),
        ),
        make_box(b"vttc", &make_box(b"payl", b"Single fragment plain cue")),
    ]
    .concat();

    [
        ftyp_box(),
        make_box(b"moov", &moov_payload),
        make_box(b"moof", &moof_payload),
        make_box(b"mdat", &mdat_payload),
    ]
    .concat()
}

fn make_box(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut data = Vec::with_capacity(8 + payload.len());
    data.extend_from_slice(&((payload.len() + 8) as u32).to_be_bytes());
    data.extend_from_slice(kind);
    data.extend_from_slice(payload);
    data
}

fn ftyp_box() -> Vec<u8> {
    make_box(b"ftyp", &[0; 20])
}

fn styp_box() -> Vec<u8> {
    make_box(b"styp", &[0; 16])
}

fn mdhd_payload(timescale: u32) -> Vec<u8> {
    [
        vec![0, 0, 0, 0],
        0u32.to_be_bytes().to_vec(),
        0u32.to_be_bytes().to_vec(),
        timescale.to_be_bytes().to_vec(),
        0u32.to_be_bytes().to_vec(),
        0u32.to_be_bytes().to_vec(),
    ]
    .concat()
}

fn stsd_payload(header: &str) -> Vec<u8> {
    let mut sample_entry_payload = vec![0; 6];
    sample_entry_payload.extend_from_slice(&1u16.to_be_bytes());
    sample_entry_payload.extend_from_slice(&make_box(b"vttC", header.as_bytes()));

    [
        vec![0, 0, 0, 0],
        1u32.to_be_bytes().to_vec(),
        make_box(b"wvtt", &sample_entry_payload),
    ]
    .concat()
}

fn tfdt_payload(base_decode_time: u32) -> Vec<u8> {
    [vec![0, 0, 0, 0], base_decode_time.to_be_bytes().to_vec()].concat()
}

fn trun_payload(samples: &[(u32, i32)]) -> Vec<u8> {
    let mut payload = vec![1, 0x00, 0x09, 0x00];
    payload.extend_from_slice(&(samples.len() as u32).to_be_bytes());

    for (duration, composition_offset) in samples {
        payload.extend_from_slice(&duration.to_be_bytes());
        payload.extend_from_slice(&composition_offset.to_be_bytes());
    }

    payload
}
