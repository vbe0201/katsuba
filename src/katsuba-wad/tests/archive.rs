use katsuba_wad::{Archive, ArchiveError, Inflater};

#[test]
fn open_mmap() -> Result<(), ArchiveError> {
    Archive::open_mmap("tests/data/Test.wad").map(|_| ())
}

#[test]
fn open_heap() -> Result<(), ArchiveError> {
    Archive::open_heap("tests/data/Test.wad").map(|_| ())
}

#[test]
fn uncompressed() -> Result<(), ArchiveError> {
    let archive = Archive::open_heap("tests/data/Test.wad")?;

    // Extract the raw file contents which should be uncompressed.
    let file = archive.file_raw("uncompressed.mp3").unwrap();
    assert!(!file.compressed);

    assert_eq!(
        archive.file_contents(file).unwrap(),
        &[117, 110, 99, 111, 109, 112, 114, 101, 115, 115, 101, 100, 32, 100, 97, 116, 97, 10]
    );

    Ok(())
}

#[test]
fn subdir() -> Result<(), ArchiveError> {
    let archive = Archive::open_heap("tests/data/Test.wad")?;
    let mut inflater = Inflater::new();

    let file = archive.file_raw("subdir/subdir_text1.txt").unwrap();
    assert!(file.compressed);

    let data = inflater.decompress(
        archive.file_contents(file).unwrap(),
        file.uncompressed_size as _,
    )?;
    assert_eq!(data, b"this is subdir text1\n");

    Ok(())
}

#[test]
fn two_files() -> Result<(), ArchiveError> {
    let archive = Archive::open_heap("tests/data/Test.wad")?;

    let text1 = archive.file_raw("text1.txt").unwrap();
    assert!(text1.compressed);

    let subdir = archive.file_raw("subdir/subdir_text1.txt").unwrap();
    assert!(subdir.compressed);

    assert_ne!(archive.file_contents(text1), archive.file_contents(subdir));

    Ok(())
}

#[test]
fn inflate_twice() -> Result<(), ArchiveError> {
    let archive = Archive::open_heap("tests/data/Test.wad")?;
    let mut inflater = Inflater::new();

    let file = archive.file_raw("text1.txt").unwrap();
    assert!(file.compressed);

    let a = inflater
        .decompress(
            archive.file_contents(file).unwrap(),
            file.uncompressed_size as _,
        )?
        .to_owned();
    let b = inflater
        .decompress(
            archive.file_contents(file).unwrap(),
            file.uncompressed_size as _,
        )?
        .to_owned();

    assert_eq!(a, b);

    Ok(())
}
