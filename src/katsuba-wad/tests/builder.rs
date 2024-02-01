use katsuba_wad::{Archive, ArchiveBuilder, Inflater};
use tempfile::NamedTempFile;

#[test]
fn build_and_extract() {
    let temp = NamedTempFile::new().unwrap();
    let (file, path) = temp.into_parts();

    let mut builder = ArchiveBuilder::new(2, 0, &path).unwrap();
    builder
        .add_file_compressed("a/b/x.txt", b"does this work?")
        .unwrap();
    builder.add_file("test.txt", b"it does!").unwrap();
    builder.finish().unwrap();

    let archive = Archive::heap(file).unwrap();
    let mut inflater = Inflater::new();

    let a = archive.file_raw("a/b/x.txt").unwrap();
    assert!(a.compressed);
    assert_eq!(
        inflater.decompress(archive.file_contents(a).unwrap(), a.uncompressed_size as _,),
        Ok(&b"does this work?"[..])
    );

    let b = archive.file_raw("test.txt").unwrap();
    assert!(!b.compressed);
    assert_eq!(archive.file_contents(b), Some(&b"it does!"[..]));
}
