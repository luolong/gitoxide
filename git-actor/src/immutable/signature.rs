mod decode {
    use btoi::btoi;

    use crate::{immutable::Signature, Sign, Time};
    use bstr::ByteSlice;
    use nom::{
        branch::alt,
        bytes::complete::{tag, take, take_until, take_while_m_n},
        character::is_digit,
        combinator::map_res,
        error::{context, ContextError, FromExternalError, ParseError},
        sequence::{terminated, tuple},
        IResult,
    };

    pub(crate) const SPACE: &[u8] = b" ";

    /// Parse a signature from the bytes input `i` using `nom`.
    pub fn signature<
        'a,
        E: ParseError<&'a [u8]>
            + ContextError<&'a [u8]>
            + FromExternalError<&'a [u8], btoi::ParseIntegerError>
            + std::fmt::Debug,
    >(
        i: &'a [u8],
    ) -> IResult<&'a [u8], Signature<'a>, E> {
        let (i, (name, email, time, tzsign, hours, minutes)) = context(
            "<name> <<email>> <timestamp> <+|-><HHMM>",
            tuple((
                context("<name>", terminated(take_until(&b" <"[..]), take(2usize))),
                context("<email>", terminated(take_until(&b"> "[..]), take(2usize))),
                context(
                    "<timestamp>",
                    map_res(terminated(take_until(SPACE), take(1usize)), btoi::<u32>),
                ),
                context("+|-", alt((tag(b"-"), tag(b"+")))),
                context("HH", map_res(take_while_m_n(2usize, 2, is_digit), btoi::<i32>)),
                context("MM", map_res(take_while_m_n(2usize, 2, is_digit), btoi::<i32>)),
            )),
        )(i)?;

        debug_assert!(tzsign[0] == b'-' || tzsign[0] == b'+', "parser assure it's +|- only");
        let sign = if tzsign[0] == b'-' { Sign::Minus } else { Sign::Plus }; //
        let offset = (hours * 3600 + minutes * 60) * if sign == Sign::Minus { -1 } else { 1 };

        Ok((
            i,
            Signature {
                name: name.as_bstr(),
                email: email.as_bstr(),
                time: Time { time, offset, sign },
            },
        ))
    }

    #[cfg(test)]
    mod tests {
        mod parse_signature {
            use crate::{immutable::signature, immutable::Signature, Sign, Time};
            use bstr::{BStr, ByteSlice};
            use nom::{error::VerboseError, IResult};

            fn decode(i: &[u8]) -> IResult<&[u8], Signature<'_>, nom::error::VerboseError<&[u8]>> {
                signature::decode(i)
            }

            fn to_bstr_err(err: nom::Err<VerboseError<&[u8]>>) -> VerboseError<&BStr> {
                let err = match err {
                    nom::Err::Error(err) | nom::Err::Failure(err) => err,
                    nom::Err::Incomplete(_) => unreachable!("not a streaming parser"),
                };
                VerboseError {
                    errors: err.errors.into_iter().map(|(i, v)| (i.as_bstr(), v)).collect(),
                }
            }

            fn signature(
                name: &'static str,
                email: &'static str,
                time: u32,
                sign: Sign,
                offset: i32,
            ) -> Signature<'static> {
                Signature {
                    name: name.as_bytes().as_bstr(),
                    email: email.as_bytes().as_bstr(),
                    time: Time { time, offset, sign },
                }
            }

            #[test]
            fn tz_minus() {
                assert_eq!(
                    decode(b"Sebastian Thiel <byronimo@gmail.com> 1528473343 -0230")
                        .expect("parse to work")
                        .1,
                    signature("Sebastian Thiel", "byronimo@gmail.com", 1528473343, Sign::Minus, -9000)
                );
            }

            #[test]
            fn tz_plus() {
                assert_eq!(
                    decode(b"Sebastian Thiel <byronimo@gmail.com> 1528473343 +0230")
                        .expect("parse to work")
                        .1,
                    signature("Sebastian Thiel", "byronimo@gmail.com", 1528473343, Sign::Plus, 9000)
                );
            }

            #[test]
            fn negative_offset_0000() {
                assert_eq!(
                    decode(b"Sebastian Thiel <byronimo@gmail.com> 1528473343 -0000")
                        .expect("parse to work")
                        .1,
                    signature("Sebastian Thiel", "byronimo@gmail.com", 1528473343, Sign::Minus, 0)
                );
            }

            #[test]
            fn empty_name_and_email() {
                assert_eq!(
                    decode(b" <> 12345 -1215").expect("parse to work").1,
                    signature("", "", 12345, Sign::Minus, -44100)
                );
            }

            #[test]
            fn invalid_signature() {
                assert_eq!(
                        decode(b"hello < 12345 -1215")
                            .map_err(to_bstr_err)
                            .expect_err("parse fails as > is missing")
                            .to_string(),
                        "Parse error:\nTakeUntil at:  12345 -1215\nin section '<email>', at:  12345 -1215\nin section '<name> <<email>> <timestamp> <+|-><HHMM>', at: hello < 12345 -1215\n"
                    );
            }

            #[test]
            fn invalid_time() {
                assert_eq!(
                        decode(b"hello <> abc -1215")
                            .map_err(to_bstr_err)
                            .expect_err("parse fails as > is missing")
                            .to_string(),
                        "Parse error:\nMapRes at: abc -1215\nin section '<timestamp>', at: abc -1215\nin section '<name> <<email>> <timestamp> <+|-><HHMM>', at: hello <> abc -1215\n"
                    );
            }
        }
    }
}
pub use decode::signature as decode;
