/// This is a bodge
///
/// It is a copy of `double()` from nom.
///
/// The only difference is that it does not parse the optional
/// exponent part of the float.
///
/// This allows parameters blocks with a "E" tag to be processes correctly.
use nom::character::complete::{char, digit1};
use nom::{
    branch::alt,
    combinator::{map, opt, recognize},
    error::{ErrorKind, ParseError},
    sequence::pair,
    AsBytes, AsChar, Compare, IResult, Input, Offset, ParseTo, Parser,
};

/// Recognizes floating point number in text format and returns a f64.
///
/// *Complete version*: Can parse until the end of input.
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed};
/// # use nom::Needed::Size;
/// use nom::number::complete::double;
///
/// let parser = |s| {
///   double(s)
/// };
///
/// assert_eq!(parser("11e-1"), Ok(("", 1.1)));
/// assert_eq!(parser("123E-02"), Ok(("", 1.23)));
/// assert_eq!(parser("123K-01"), Ok(("K-01", 123.0)));
/// assert_eq!(parser("abc"), Err(Err::Error(("abc", ErrorKind::Float))));
/// ```
pub fn double_no_exponent<T, E: ParseError<T>>(input: T) -> IResult<T, f64, E>
where
    T: AsBytes + Clone + Input + Offset + ParseTo<f64> + Compare<&'static str>,
    <T as Input>::Item: AsChar,
    <T as Input>::Iter: Clone,
    T: for<'a> Compare<&'a [u8]>,
{
    /*
    let (i, (sign, integer, fraction, exponent)) = recognize_float_parts(input)?;

    let mut float: f64 = minimal_lexical::parse_float(
      integer.as_bytes().iter(),
      fraction.as_bytes().iter(),
      exponent,
    );
    if !sign {
      float = -float;
    }

    Ok((i, float))
        */
    let (i, s) = recognize_float_or_exceptions(input)?;
    match s.parse_to() {
        Some(f) => Ok((i, f)),
        None => Err(nom::Err::Error(E::from_error_kind(
            i,
            nom::error::ErrorKind::Float,
        ))),
    }
}

// workaround until issues with minimal-lexical are fixed
#[doc(hidden)]
pub fn recognize_float_or_exceptions<T, E: ParseError<T>>(input: T) -> IResult<T, T, E>
where
    T: Clone + Offset + Input + Compare<&'static str>,
    <T as Input>::Item: AsChar,
{
    alt((
        |i: T| {
            recognize_float::<_, E>(i.clone()).map_err(|e| match e {
                nom::Err::Error(_) => nom::Err::Error(E::from_error_kind(i, ErrorKind::Float)),
                nom::Err::Failure(_) => nom::Err::Failure(E::from_error_kind(i, ErrorKind::Float)),
                nom::Err::Incomplete(needed) => nom::Err::Incomplete(needed),
            })
        },
        |i: T| {
            nom::bytes::complete::tag_no_case::<_, _, E>("nan")(i.clone())
                .map_err(|_| nom::Err::Error(E::from_error_kind(i, ErrorKind::Float)))
        },
        |i: T| {
            nom::bytes::complete::tag_no_case::<_, _, E>("infinity")(i.clone())
                .map_err(|_| nom::Err::Error(E::from_error_kind(i, ErrorKind::Float)))
        },
        |i: T| {
            nom::bytes::complete::tag_no_case::<_, _, E>("inf")(i.clone())
                .map_err(|_| nom::Err::Error(E::from_error_kind(i, ErrorKind::Float)))
        },
    ))
    .parse(input)
}

/// Recognizes floating point number in a byte string and returns the corresponding slice.
///
/// *Complete version*: Can parse until the end of input.
///
/// ```rust
/// # use nom::{Err, error::ErrorKind, Needed};
/// # use nom::Needed::Size;
/// use nom::number::complete::recognize_float;
///
/// let parser = |s| {
///   recognize_float(s)
/// };
///
/// assert_eq!(parser("11e-1"), Ok(("", "11e-1")));
/// assert_eq!(parser("123E-02"), Ok(("", "123E-02")));
/// assert_eq!(parser("123K-01"), Ok(("K-01", "123")));
/// assert_eq!(parser("abc"), Err(Err::Error(("abc", ErrorKind::Char))));
/// ```
#[rustfmt::skip]
pub fn recognize_float<T, E:ParseError<T>>(input: T) -> IResult<T, T, E>
where
  T: Clone + Offset + Input,
  <T as Input>::Item: AsChar,
{
  recognize((
    opt(alt((char('+'), char('-')))),
      alt((
        map((digit1, opt(pair(char('.'), opt(digit1)))), |_| ()),
        map((char('.'), digit1), |_| ())
      )),
  )).parse(input)
}

// #[rustfmt::skip]
// pub fn recognize_float<T, E:ParseError<T>>(input: T) -> IResult<T, T, E>
// where
//   T: Clone + Offset,
//   T: Input,
//   <T as Input>::Item: AsChar,
// {
//   recognize((
//     opt(alt((char('+'), char('-')))),
//       alt((
//         map((digit1, opt(pair(char('.'), opt(digit1)))), |_| ()),
//         map((char('.'), digit1), |_| ())
//       )),
//       opt((
//         alt((char('e'), char('E'))),
//         opt(alt((char('+'), char('-')))),
//         cut(digit1)
//       ))
//   )).parse(input)
// }
