use std::io;

use katsuba_bit_buf::BitReader;

#[test]
fn read_primitives() -> io::Result<()> {
    let mut buf = BitReader::new(&[0xDE, 0xC0, 0xAD, 0xDE]);

    assert_eq!(buf.remaining_bits(), 32);
    assert_eq!(buf.refill_bits(), 32);

    assert!(matches!(buf.peek(u16::BITS)?, 0xC0DE));
    buf.consume(u16::BITS)?;
    assert_eq!(buf.remaining_bits(), 16);

    assert!(matches!(buf.peek(u8::BITS)?, 0xAD));
    buf.consume(u8::BITS)?;
    assert!(matches!(buf.peek(u8::BITS)?, 0xDE));
    buf.consume(u8::BITS)?;

    Ok(())
}

#[test]
fn read_bits_and_alignment() -> io::Result<()> {
    let mut buf = BitReader::new(&[1, 2, 3, 4]);

    assert_eq!(buf.refill_bits(), 32);

    assert!(matches!(buf.peek(1)?, 1));
    buf.consume(1)?;
    assert!(matches!(buf.peek(1)?, 0));
    buf.consume(1)?;
    assert_eq!(buf.remaining_bits(), 30);

    buf.realign_to_byte();
    assert_eq!(buf.refill_bits(), 24);

    assert!(matches!(buf.peek(u8::BITS)?, 2));
    buf.consume(u8::BITS)?;
    assert_eq!(buf.remaining_bits(), 16);

    assert_eq!(buf.refill_bits(), 16);

    assert!(matches!(buf.peek(1)?, 1));
    buf.consume(1)?;
    assert!(matches!(buf.peek(1)?, 1));
    buf.consume(1)?;

    buf.realign_to_byte();

    assert_eq!(buf.read_bytes(1)?, &[4]);
    assert_eq!(buf.remaining_bits(), 0);

    Ok(())
}
