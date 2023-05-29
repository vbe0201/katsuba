use kobold_bit_buf::BitReader;

#[test]
fn read_primitives() {
    let mut buf = BitReader::new_borrowed(&[0xDE, 0xC0, 0xAD, 0xDE]);

    assert_eq!(buf.remaining_bits(), 32);

    assert!(matches!(buf.u16(), 0xC0DE));
    assert_eq!(buf.remaining_bits(), 16);

    assert!(matches!(buf.u8(), 0xAD));
    assert!(matches!(buf.u8(), 0xDE));
}

#[test]
fn read_bits_and_alignment() {
    let mut buf = BitReader::new_borrowed(&[1, 2, 3, 4]);

    assert_eq!(buf.refill_bits(), 32);

    assert!(matches!(buf.bool(), true));
    assert!(matches!(buf.bool(), false));
    assert_eq!(buf.remaining_bits(), 30);

    buf.invalidate_and_realign_ptr();

    assert!(matches!(buf.u8(), 2));
    assert_eq!(buf.remaining_bits(), 16);
    
    assert_eq!(buf.refill_bits(), 16);

    assert!(matches!(buf.bool(), true));
    assert!(matches!(buf.bool(), true));

    buf.invalidate_and_realign_ptr();

    assert_eq!(buf.read_bytes(1), &[4]);
    assert_eq!(buf.remaining_bits(), 0);
}
