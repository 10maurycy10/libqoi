mod decoder;
mod encoder;
mod shared;

pub use decoder::decode_qoi;
pub use encoder::encode_qoi;
pub use shared::read_header;
pub use shared::Header;
pub use shared::Part;

#[cfg(test)]
mod tests {
    use crate::shared::Header;

    use crate::shared::read_header;
    #[test]
    fn can_read_header() {
        assert_eq!(
            read_header(&[
                b'q', b'o', b'i', b'f', 0x00, 0x00, 0x00, 15, 0x00, 0x00, 0x00, 24, 3, 1, 0xFF
            ]),
            Some((
                Header {
                    height: 24,
                    width: 15,
                    channel_count: 3,
                    color_space: 1
                },
                &[0xff][..]
            ))
        );
    }
}
