// Common processor test utilities and shared test data

use encoding_rs::WINDOWS_1252;

mod async_processing;
mod sync_processing;

pub const MUSICAL_NOTE_EXAMPLE: &str = r#"1
00:01:00,000 --> 00:01:01,000
#TestData

2
00:02:00,000 --> 00:02:01,000
#TestData#

3
00:03:00,000 --> 00:03:01,000
# #TestData #

4
00:04:00,000 --> 00:04:01,000
# Song Lyrics #

5
00:05:00,000 --> 00:05:01,000
We are #1!

6
00:06:00,000 --> 00:06:01,000
# <i>Song Lyrics</i>

7
00:07:00,000 --> 00:07:01,000
# Song Lyrics
On two separate lines #

8
00:08:00,000 --> 00:08:01,000
#1 Radio Station

9
00:09:00,000 --> 00:09:01,000
ABCD FM
#1 Radio Station

10
00:10:00,000 --> 00:10:01,000
#One Radio Station

11
00:11:00,000 --> 00:11:01,000
♪ <i>Fire</i>♪

12
00:12:00,000 --> 00:12:01,000
*Schnaub*

13
00:13:00,000 --> 00:13:01,000
* Schnaub *

14
00:14:00,000 --> 00:14:01,000
♫ Thunder"#;

pub const ADDING_LINE_BREAKS_EXAMPLE: &str = r#"1
00:01:00,000 --> 00:01:01,000
It's chocolate.Hmm?

2
00:02:00,000 --> 00:02:01,000
We can't just leave him.He's already gone.

3
00:03:26,800 --> 00:03:31,200
- Test. Mr.Teufel...
- Test..."#;

pub const ELIPSES_FIXING_EXAMPLE: &str = r#"1
00:13:00,000 --> 00:13:01,000
..noooooooooooooo..........

2
00:14:00,000 --> 00:14:01,000
<i>Stop this.................</i>"#;

pub const TAG_CORRECTIONS_EXAMPLE: &str = r#"1
00:15:00,000 --> 00:15:01,000
<i>    Test</i> <i>line1</i>
<i>Test </i>line2

2
00:16:00,000 --> 00:16:01,000
{\an3}{\an8}{\an8}<i><i>Test line1
Test line2</i>

3
00:17:00,000 --> 00:17:01,000
<b>test</b>

4
00:18:00,000 --> 00:18:01,000
<i>   
test
</i>"#;

pub const GAP_REMOVAL_EXAMPLE: &str = r#"1
00:19:00,000 --> 00:19:00,100
remove 2 frame gap between this

2
00:19:00,183 --> 00:19:01,000
and that line"#;

pub const SPACE_REMOVAL_EXAMPLE: &str = r#"
1
00:22:00,000 --> 00:22:01,000
<i>SOMETHING:</i> <i>
Synthetic test.</i> <i>
Definitely not real.</i>
"#;

pub const SPACES_AFTER_HYPHENS_EXAMPLE: &str = r#"1
00:23:00,000 --> 00:23:01,000
-Well.
-$5000?"#;

pub const INVALID_TIMESTAMP_EXAMPLE: &str = r#"1
27:27:00,000 --> 27:27:01,000
Always. Run. Tests.

2
28:27:00,000 --> 28:27:01,000
Really."#;

pub const OVERLAPPING_TIME_EXAMPLE: &str = r#"1
00:00:00,000 --> 00:00:00,105
this line should end at 104

2
00:00:00,105 --> 00:00:01,000
and that line should end start at 105"#;

pub const DUPE_ALIGNMENT_EXAMPLE: &str = r#"1
00:02:05.289 --> 00:02:07.416
{\an8}I'm only nineteen

2
00:02:05.289 --> 00:02:07.416
{\an8}but my mind is old"#;

pub const DUPLICATE_EXAMPLE: &str = r#"1
00:01:00,000 --> 00:01:02,000
First subtitle

2  
00:01:00,000 --> 00:01:02,000
First subtitle

3
00:01:03,000 --> 00:01:04,000
Second subtitle"#;

pub const ITERATION_EXAMPLE: &str = r#"1
00:01:00,000 --> 00:01:01,000
First line

2
00:01:02,000 --> 00:01:03,000
Second line

3
00:01:04,000 --> 00:01:05,000
Third line"#;

pub const SDH_EXAMPLE: &str = r#"1
00:00:11,803 --> 00:00:13,346
RADIO ANNOUNCER:
<i>"W" who?</i>

2
00:00:40,749 --> 00:00:42,375
- ♪ Hey, boo ♪
- ♪ Hey, boo ♪

3
00:00:55,931 --> 00:00:58,134
[ Maker's "Hold'em" playing ]

4
00:00:58,934 --> 00:01:06,567
♪

5
00:00:59,292 --> 00:01:01,561
- [shouting]
- [continuous gunfire]

6
00:01:09,653 --> 00:01:11,822
It's zoo time!
[ Kids cheering ]

7
00:01:29,881 --> 00:01:31,132
{\an8}(MYSTERIOUS MUSIC PLAYING) Spooky!

8
00:01:33,968 --> 00:01:35,387
[John] Hmm?
<i>(Alice) Hello!</i>

9
00:01:40,016 --> 00:01:41,685
- I did on magnets this summer.
- (ELECTRICITY ZAPS)

10
00:01:41,685 --> 00:01:42,769
- Boo!
- STUDENT: No, thanks.

11
00:01:43,685 --> 00:01:44,769
>> SO THIS IS MY HOME OFFICE
HERE. IN THIS OFFICE ARE A LOT

12
00:01:45,685 --> 00:01:46,769
>>"#;

// SDH test constants for extra regex testing
pub const SDH_EXTRA_REGEX_TEST: &str = r#"1
00:00:01,000 --> 00:00:02,000
This is a TEST line.

2
00:00:03,000 --> 00:00:04,000
Normal line."#;

pub const SDH_COMPREHENSIVE_TEST: &str = r#"1
00:00:01,000 --> 00:00:02,000
This is an AD for www.example.com [MUSIC]

2
00:00:03,000 --> 00:00:04,000
Normal subtitle content

3  
00:00:05,000 --> 00:00:06,000
Another AD with [MUSIC] playing"#;

pub const SDH_EDGE_CASE_TEST: &str = r#"1
00:00:01,000 --> 00:00:02,000
This    is   an   Advertisement  at  12:34

2
00:00:03,000 --> 00:00:04,000
Normal   subtitle   with   PROMO"#;

pub const SDH_EMPTY_CONTENT_TEST: &str = r#"1
00:00:01,000 --> 00:00:02,000
This will be removed

2
00:00:03,000 --> 00:00:04,000
This too"#;

pub const SDH_BLEEP_TEST: &str = r#"1
00:00:01,000 --> 00:00:02,000
[bleep]

2
00:00:03,000 --> 00:00:04,000
(bleep)

3
00:00:05,000 --> 00:00:06,000
[crowd cheering] Move!"#;

pub const SINGLE_CHARACTER_LITERAL_PATTERN_EXAMPLE: &str = r#"1
00:00:01,000 --> 00:00:02,000
A

2
00:00:03,000 --> 00:00:04,000
[A-Za-z0-9]

3
00:00:05,000 --> 00:00:06,000
B"#;

pub fn misdecode_utf8_as_windows_1252(text: &str) -> String {
    let (decoded, _, had_errors) = WINDOWS_1252.decode(text.as_bytes());
    assert!(
        !had_errors,
        "The test helper should only produce deterministic Windows-1252 mojibake"
    );
    decoded.into_owned()
}
