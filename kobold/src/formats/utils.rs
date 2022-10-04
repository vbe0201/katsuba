use binrw::{
    io::{Read, Seek},
    BinRead, BinResult, Error, ReadOptions, VecArgs,
};

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
