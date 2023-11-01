use std::io;

use katsuba_bit_buf::BitWriter;

#[test]
fn write_primitives() -> io::Result<()> {
    let mut writer = BitWriter::new();

    writer.offer(0xFF, u8::BITS)?;
    writer.offer(0xDEAD, u16::BITS)?;
    writer.offer(0xFF, u8::BITS)?;
    writer.commit();

    assert_eq!(writer.view(), &[0xFF, 0xAD, 0xDE, 0xFF]);

    Ok(())
}

#[test]
fn write_length_prefix() -> io::Result<()> {
    let mut writer = BitWriter::new();

    writer.length_prefixed(|w| w.offer(0xDEADBEEF, 31))?;
    writer.offer(1, 1)?;
    writer.commit();

    assert_eq!(
        writer.view(),
        &[0x3F, 0x00, 0x00, 0x00, 0xEF, 0xBE, 0xAD, 0xDE]
    );

    Ok(())
}

#[test]
fn write_bytes_and_alignment() -> io::Result<()> {
    let mut writer = BitWriter::new();

    writer.offer(1, 1)?;
    assert_eq!(writer.written_bits(), 1);

    writer.realign_to_byte();

    writer.offer(3, u8::BITS)?;
    writer.commit();
    assert_eq!(writer.written_bits(), 16);

    writer.offer(0, 1)?;
    writer.offer(1, 1)?;

    writer.realign_to_byte();

    writer.write_bytes(&[4, 5]);

    assert_eq!(writer.view(), &[1, 3, 2, 4, 5]);

    Ok(())
}
