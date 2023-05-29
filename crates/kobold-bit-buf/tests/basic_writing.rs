use kobold_bit_buf::BitWriter;

#[test]
fn write_primitives() {
    let mut writer = BitWriter::new();

    writer.u8(0xFF);
    writer.u16(0xDEAD);
    writer.u8(0xFF);

    assert_eq!(writer.view(), &[0xFF, 0xAD, 0xDE, 0xFF]);
}

#[test]
fn write_length_prefix() {
    let mut writer = BitWriter::new();

    let len = writer.mark_len();
    writer.u32(0xDEADBEEF);
    writer.commit_len(len);

    assert_eq!(writer.view(), &[0x40, 0x00, 0x00, 0x00, 0xEF, 0xBE, 0xAD, 0xDE]);
}

#[test]
fn write_bytes_and_alignment() {
    let mut writer = BitWriter::new();

    writer.bool(true);
    assert_eq!(writer.len(), 1);

    writer.realign_to_byte();

    writer.u8(3);
    assert_eq!(writer.len(), 16);

    writer.bool(false);
    writer.bool(true);

    writer.realign_to_byte();

    writer.write_bytes(&[4, 5]);

    assert_eq!(writer.view(), &[1, 3, 2, 4, 5]);
}
