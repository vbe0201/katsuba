use binrw::{
    io::{Read, Seek},
    BinRead, BinResult, Error, ReadOptions, VecArgs,
};
use num_traits::AsPrimitive;

pub fn parse_string<R: Read + Seek>(
    reader: &mut R,
    options: &ReadOptions,
    (len,): (usize,),
) -> BinResult<String> {
    let pos = reader.stream_position()?;
    let data = Vec::read_options(reader, options, VecArgs::builder().count(len).finalize())?;

    String::from_utf8(data).map_err(|e| Error::Custom {
        pos,
        err: Box::new(e),
    })
}

pub fn parse_string_vec<T: BinRead<Args = ()> + AsPrimitive<usize>, R: Read + Seek>(
    reader: &mut R,
    options: &ReadOptions,
    (count,): (usize,),
) -> BinResult<Vec<String>> {
    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
        // Read the length prefix of the string data.
        let len = T::read_options(reader, options, ())?;

        // Read the actual data.
        result.push(parse_string(reader, options, (len.as_(),))?);
    }

    Ok(result)
}
