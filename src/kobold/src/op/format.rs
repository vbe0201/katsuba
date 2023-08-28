use std::io;

use kobold_object_property::value::*;

use super::display::*;

pub fn value<W: io::Write>(writer: &mut W, v: Value) -> io::Result<()> {
    // The root object does not require any indentation.
    value_at(writer, 0, false, v)
}

fn value_at<W: io::Write>(writer: &mut W, depth: usize, nl: bool, v: Value) -> io::Result<()> {
    match v {
        Value::Empty => empty(writer),

        Value::Unsigned(i) => write!(writer, "{i}"),
        Value::Signed(i) => write!(writer, "{i}"),
        Value::Float(i) => write!(writer, "{i}"),
        Value::Bool(b) => write!(writer, "{b}"),

        Value::String(s) => string(writer, &s),
        Value::WString(s) => wstring(writer, &s),

        Value::Enum(v) => write!(writer, "{v}"), // TODO: string repr?

        Value::List(v) => list(writer, depth, v),
        Value::Object(obj) => object(writer, depth, nl, obj),

        Value::Color(v) => color(writer, v),
        Value::Vec3(v) => vec3(writer, v),
        Value::Quat(v) => quat(writer, v),
        Value::Euler(v) => euler(writer, v),
        Value::Mat3x3(v) => matrix(writer, *v),

        Value::Point { xy: v } | Value::Size { wh: v } => {
            write!(writer, "(")?;
            value_at(writer, depth, nl, v.0)?;
            write!(writer, ", ")?;
            value_at(writer, depth, nl, v.1)?;
            write!(writer, ")")
        }
        Value::Rect { inner: v } => {
            write!(writer, "(")?;
            value_at(writer, depth, nl, v.0)?;
            write!(writer, ", ")?;
            value_at(writer, depth, nl, v.1)?;
            write!(writer, ", ")?;
            value_at(writer, depth, nl, v.2)?;
            write!(writer, ", ")?;
            value_at(writer, depth, nl, v.3)?;
            write!(writer, ")")
        }
    }
}

pub fn empty<W: io::Write>(writer: &mut W) -> io::Result<()> {
    write!(writer, "null")
}

pub fn color<W: io::Write>(writer: &mut W, v: Color) -> io::Result<()> {
    let Color { r, g, b, a } = v;
    write!(writer, "\"#{r:02X}{g:02X}{b:02X}{a:02X}\"")
}

pub fn vec3<W: io::Write>(writer: &mut W, v: Vec3) -> io::Result<()> {
    let Vec3 { x, y, z } = v;
    write!(writer, "[{x}, {y}, {z}]")
}

pub fn quat<W: io::Write>(writer: &mut W, v: Quaternion) -> io::Result<()> {
    let Quaternion { x, y, z, w } = v;
    write!(writer, "[{x}, {y}, {z}, {w}]")
}

pub fn euler<W: io::Write>(writer: &mut W, v: Euler) -> io::Result<()> {
    let Euler { pitch, roll, yaw } = v;
    write!(writer, "[{pitch}, {roll}, {yaw}]")
}

pub fn matrix<W: io::Write>(writer: &mut W, v: Matrix) -> io::Result<()> {
    let Matrix { i, j, k } = v;
    write!(writer, "[{i:?}, {j:?}, {k:?}]")
}

pub fn string<W: io::Write>(writer: &mut W, v: &[u8]) -> io::Result<()> {
    write!(writer, "\"{}\"", CxxStr(v))
}

pub fn wstring<W: io::Write>(writer: &mut W, v: &[u16]) -> io::Result<()> {
    write!(writer, "\"{}\"", CxxWStr(v))
}

pub fn list<W: io::Write>(writer: &mut W, depth: usize, v: List) -> io::Result<()> {
    // If the list is empty, the literal doesn't need indentation.
    if v.is_empty() {
        return write!(writer, "[]");
    }

    // Recursively format all list entries.
    writeln!(writer, "[")?;
    for e in v {
        value_at(writer, depth + 1, true, e)?;
        writeln!(writer, ",")?;
    }
    write!(writer, "{:indent$}]", "", indent = depth * 4)
}

pub fn object<W: io::Write>(writer: &mut W, depth: usize, nl: bool, v: Object) -> io::Result<()> {
    // When we're starting in a new line, indent properly.
    if nl {
        write!(writer, "{:indent$}", "", indent = depth * 4)?;
    }

    writeln!(writer, "{{")?;
    for (k, v) in v {
        write!(writer, "{:indent$}\"{k}\": ", "", indent = (depth + 1) * 4)?;
        value_at(writer, depth + 1, false, v)?;
        writeln!(writer, ",")?;
    }
    write!(writer, "{:indent$}}}", "", indent = depth * 4)
}
