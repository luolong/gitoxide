use cargo_smart_release::{changelog, changelog::Section, ChangeLog};
use git_repository::bstr::ByteSlice;

use crate::Result;
use cargo_smart_release::changelog::section;

#[test]
fn all_section_types_round_trips_lossy() -> Result {
    let mut log = ChangeLog {
        sections: vec![
            Section::Verbatim {
                text: "# Changelog\n\nmy very own header\n\n".into(),
                generated: false,
            },
            Section::Release {
                heading_level: 2,
                date: None,
                name: changelog::Version::Unreleased,
                segments: Vec::new(),
                unknown: "hello\nworld\n".into(),
            },
            Section::Release {
                heading_level: 4,
                date: Some(time::OffsetDateTime::from_unix_timestamp(0)?),
                name: changelog::Version::Semantic("1.0.2-beta.2".parse()?),
                segments: vec![
                    section::Segment::User {
                        text: "* hello world\n\tthis\n".into(),
                    },
                    section::Segment::Clippy(Some(section::ThanksClippy { count: 42 })),
                ],
                unknown: String::new(),
            },
        ],
    };
    for _round in 0..1 {
        // NOTE: we can't run this a second time as the statistical information will be gone (it was never parsed back)
        let mut buf = Vec::<u8>::new();
        log.write_to(&mut buf)?;
        let md = buf.to_str()?;
        insta::assert_snapshot!(md);

        let parsed_log = ChangeLog::from_markdown(md);
        assert_eq!(parsed_log, log, "we must be able to parse the exact input back");
        log = parsed_log;
    }
    Ok(())
}