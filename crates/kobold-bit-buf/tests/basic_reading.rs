use kobold_bit_buf::BitReader;

#[test]
fn primitives() {
    let mut buf = BitReader::new(vec![0xDE, 0xC0, 0xAD, 0xDE]);

    assert_eq!(buf.len(), buf.remaining());

    assert!(matches!(buf.u16(), Ok(0xC0DE)));
    assert_eq!(buf.remaining(), buf.len() / 2);

    assert!(matches!(buf.u8(), Ok(0xAD)));
    assert!(matches!(buf.u8(), Ok(0xDE)));
}

#[test]
fn read_bits_and_alignment() {
    let mut buf = BitReader::new(vec![1, 2, 3, 4]);

    assert!(matches!(buf.bool(), Ok(true)));
    assert!(matches!(buf.bool(), Ok(false)));
    assert_eq!(buf.remaining(), buf.len() - 2);

    assert!(matches!(buf.u8(), Ok(2)));
    assert_eq!(buf.remaining(), buf.len() / 2);

    assert!(matches!(buf.bool(), Ok(true)));
    assert!(matches!(buf.bool(), Ok(true)));

    assert!(matches!(buf.read_bytes(1), Ok(&[4])));
    assert_eq!(buf.remaining(), 0);
}

#[test]
fn test_no_into_inner_uaf() {
    let buf = BitReader::new(vec![1, 2, 3]);
    assert_eq!(buf.into_inner(), &[1, 2, 3]);
}
