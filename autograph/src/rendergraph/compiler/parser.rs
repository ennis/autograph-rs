use nom::*;
use std::str::from_utf8_unchecked;
use super::syntax::*;

/// Parse a single comment.
named!(pub comment,
  delimited!(sp,
             alt!(
               complete!(preceded!(tag!("//"), take_until!("\n"))) |
               complete!(delimited!(tag!("/*"), take_until!("*/"), tag!("*/"))) |
               sp
             ),
             sp)
);

/// Parse an alphanumeric separator. An alphanumeric separator is a char used to separate
/// alphanumeric tokens. For instance, "in vec3 x" contains three alphanumeric tokens, while
/// "int x = 3, y, z = 12;" contains six alphanumeric tokens ('=', ',' and ';' are separators).
///
/// Whitespace are also considered such separators.
named!(pub alphasep<&[u8], char>, peek!(one_of!(" \t\n,;:.<>{}[]()+-%*/=^?\"'")));

/// Parse a tag followed by an alphanumeric separator.
macro_rules! atag {
  ($i:expr, $s:expr) => {{
    terminated!($i, tag!($s), alphasep)
  }}
}

/// Parse several comments.
named!(pub comments, recognize!(many0!(comment)));

/// Parser rewriter, discarding whitespaces and comments.
#[macro_export]
macro_rules! bl {
  ($i:expr, $($args:tt)*) => {{
    sep!($i, comment, $($args)*)
  }}
}

// Turn a &[u8] into a &str.
#[inline]
fn bytes_to_str(bytes: &[u8]) -> &str {
    unsafe { from_utf8_unchecked(bytes) }
}

// Turn a &[u8] into a String.
#[inline]
fn bytes_to_string(bytes: &[u8]) -> String {
    bytes_to_str(bytes).to_owned()
}

/// Parse an identifier (raw version).
named!(identifier_str,
  bl!(do_parse!(
    name: verify!(take_while1!(identifier_pred), verify_identifier) >>
    (name)
  ))
);

/// Parse an identifier.
//named!(pub identifier<&[u8], syntax::Identifier>, map!(identifier_str, bytes_to_string));

#[inline]
fn identifier_pred(c: u8) -> bool {
    let ch = char::from(c);
    ch.is_alphanumeric() || ch == '_'
}

#[inline]
fn verify_identifier(s: &[u8]) -> bool {
    !char::from(s[0]).is_digit(10)
}

/// Parse a non-empty list of identifiers, delimited by comma (,).
/*named!(nonempty_identifiers<&[u8], Vec<syntax::Identifier>>,
  bl!(do_parse!(
    first: identifier >>
    rest: many0!(do_parse!(char!(',') >> i: bl!(identifier) >> (i))) >>

    ({
      let mut identifiers = rest.clone();
      identifiers.insert(0, first);
      identifiers
    })
  ))
);*/


/// Parse the void type.
named!(pub void<&[u8], ()>, value!((), atag!("void")));

/// Parse a digit that precludes a leading 0.
named!(nonzero_digit, verify!(digit, |s:&[u8]| s[0] != b'0'));

/// Parse a decimal literal string.
named!(decimal_lit_<&[u8], ()>,
  do_parse!(
    bl!(opt!(char!('-'))) >>
    nonzero_digit >>
    (())
  )
);

/// Parse a decimal literal.
named!(decimal_lit, recognize!(decimal_lit_));

#[inline]
fn is_octal(s: &[u8]) -> bool {
    s[0] == b'0' && s.iter().all(|&c| c >= b'0' && c <= b'7')
}

/// Parse an octal literal string.
named!(octal_lit_<&[u8], ()>,
  do_parse!(
    bl!(opt!(char!('-'))) >>
    verify!(digit, is_octal) >>
    (())
  )
);

/// Parse an octal literal.
named!(octal_lit, recognize!(octal_lit_));

#[inline]
fn all_hexa(s: &[u8]) -> bool {
    s.iter().all(|&c| c >= b'0' && c <= b'9' || c >= b'a' && c <= b'f' || c >= b'A' && c <= b'F')
}

#[inline]
fn alphanumeric_no_u(c: u8) -> bool {
    char::from(c).is_alphanumeric() && c != b'u' && c != b'U'
}

/// Parse an hexadecimal literal string.
named!(hexadecimal_lit_<&[u8], ()>,
  do_parse!(
    bl!(opt!(char!('-'))) >>
    alt!(tag!("0x") | tag!("0X")) >>
    verify!(take_while1!(alphanumeric_no_u), all_hexa) >>
    (())
  )
);

/// Parse an hexadecimal literal.
named!(hexadecimal_lit, recognize!(hexadecimal_lit_));

named!(integral_lit_,
  alt!(
    hexadecimal_lit |
    octal_lit |
    decimal_lit
  )
);

/// Parse a literal integral string.
named!(pub integral_lit<&[u8], i32>,
  do_parse!(
    i: integral_lit_ >>
    ({
      if i.len() > 2 {
        if i[0] == b'-' {
          let i_ = &i[1..];

          if i_.starts_with(b"0x") | i_.starts_with(b"0X") {
            -i32::from_str_radix(bytes_to_str(&i_[2..]), 16).unwrap()
          } else {
            bytes_to_str(i).parse::<i32>().unwrap()
          }
        } else if i.starts_with(b"0x") | i.starts_with(b"0X") {
          i32::from_str_radix(bytes_to_str(&i[2..]), 16).unwrap()
        } else {
          bytes_to_str(i).parse::<i32>().unwrap()
        }
      } else {
        bytes_to_str(i).parse::<i32>().unwrap()
      }
    })
  )
);

/// Parse the unsigned suffix.
named!(unsigned_suffix<&[u8], char>, alt!(char!('u') | char!('U')));

/// Parse a literal unsigned string.
named!(pub unsigned_lit<&[u8], u32>,
  do_parse!(
    i: integral_lit_ >>
    unsigned_suffix >>
    ({
      if i.len() > 2 {
        if i[0] == b'-' {
          let i_ = &i[1..];

          if i_.starts_with(b"0x") | i_.starts_with(b"0X") {
            u32::wrapping_sub(0, u32::from_str_radix(bytes_to_str(&i_[2..]), 16).unwrap())
          } else {
            bytes_to_str(i).parse::<u32>().unwrap()
          }
        } else if i.starts_with(b"0x") | i.starts_with(b"0X") {
          u32::from_str_radix(bytes_to_str(&i[2..]), 16).unwrap()
        } else {
          bytes_to_str(i).parse::<u32>().unwrap()
        }
      } else {
        bytes_to_str(i).parse::<u32>().unwrap()
      }
    })
  )
);

/// Parse a floating point suffix.
named!(float_suffix,
  alt!(
    tag!("f") |
    tag!("F")
  )
);

/// Parse a double point suffix.
named!(double_suffix,
  alt!(
    tag!("lf") |
    tag!("LF")
  )
);


/// Parse the exponent part of a floating point literal.
named!(floating_exponent<&[u8], ()>,
  do_parse!(
    alt!(char!('e') | char!('E')) >>
    opt!(alt!(char!('+') | char!('-'))) >>
    digit >>
    (())
  )
);

/// Parse the fractional constant part of a floating point literal.
named!(floating_frac<&[u8], ()>,
  alt!(
    do_parse!(char!('.') >> digit >> (())) |
    do_parse!(digit >> tag!(".") >> digit >> (())) |
    do_parse!(digit >> tag!(".") >> (()))
  )
);

/// Parse the « middle » part of a floating value – i.e. fractional and exponential parts.
named!(floating_middle, recognize!(preceded!(floating_frac, opt!(floating_exponent))));

/// Parse a float literal string.
named!(pub float_lit<&[u8], f32>,
  do_parse!(
    sign: bl!(opt!(char!('-'))) >>
    f: floating_middle >>
    opt!(float_suffix) >>

    ({
      // if the parsed data is in the accepted form ".394634…", we parse it as if it was < 0
      let n = if f[0] == b'.' {
        let mut f_ = f.to_owned();
        f_.insert(0, b'0');

        bytes_to_str(&f_).parse::<f32>().unwrap()
      } else {
        bytes_to_str(f).parse().unwrap()
      };

      // handle the sign and return
      if sign.is_some() { -n } else { n }
    })
  )
);

/// Parse a double literal string.
named!(pub double_lit<&[u8], f64>,
  do_parse!(
    sign: bl!(opt!(char!('-'))) >>
    f: floating_middle >>
    not!(float_suffix) >> // prevent from parsing 3.f ("f", Double(3.)) while it should be ("", Float(3.))
    opt!(double_suffix) >>

    ({
      // if the parsed data is in the accepted form ".394634…", we parse it as if it was < 0
      let n = if f[0] == b'.' {
        let mut f_ = f.to_owned();
        f_.insert(0, b'0');

        bytes_to_str(&f_).parse::<f64>().unwrap()
      } else {
        bytes_to_str(f).parse().unwrap()
      };

      // handle the sign and return
      if sign.is_some() { -n } else { n }
    })
  )
);

/// Parse a constant boolean.
named!(pub bool_lit<&[u8], bool>,
  alt!(
    value!(true, atag!("true")) |
    value!(false, atag!("false"))
  )
);

named!(pub p_primitive_topology<&[u8], PassDirective>,
    do_parse!(
        atag!("primitive_topology") >>
        v: alt!(
            value!(PrimitiveTopology::Triangle, atag!("triangle")) |
            value!(PrimitiveTopology::Line, atag!("line"))
        ) >>
        tag!(";") >>
        (PassDirective::PrimitiveTopology(v))
    )
);


named!(pub p_vertex_shader<&[u8], PassDirective>,
    do_parse!(
        atag!("vertex") >>
        v: identifier_str >>
        tag!(";") >>
        (PassDirective::VertexShader(v))
    )
);

