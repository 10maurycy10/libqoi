/// A QOI header, storing all the metadata
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Header {
    /// big endian
    pub height: u32,
    /// big endian
    pub width: u32,
    /// 3 = RGB
    /// 4 = RGBA
    pub channel_count: u8,
    /// 0 = SRGB
    /// 1 = SRGB - linear alpha
    /// 3 = linear all
    pub color_space: u8,
}

pub fn encode_header(head: Header, dst: &mut Vec<u8>) {
    dst.push(b'q');
    dst.push(b'o');
    dst.push(b'i');
    dst.push(b'f');
    let w = head.width.to_be_bytes();
    let h = head.height.to_be_bytes();
    dst.push(w[0]);
    dst.push(w[1]);
    dst.push(w[2]);
    dst.push(w[3]);

    dst.push(h[0]);
    dst.push(h[1]);
    dst.push(h[2]);
    dst.push(h[3]);

    dst.push(head.channel_count);
    dst.push(head.color_space);
}

/// Reads a header from data, can be used to check image size before decoding.
pub fn read_header<'a>(data: &'a [u8]) -> Option<(Header, &'a [u8])> {
    let magic = data.get(0..4)?;
    let width = data.get(4..8)?;
    let height = data.get(8..12)?;
    let channel_count = data.get(12)?;
    let color_space = data.get(13)?;
    if magic != b"qoif" {
        return None;
    } else {
        Some((
            Header {
                height: u32::from_be_bytes([height[0], height[1], height[2], height[3]]),
                width: u32::from_be_bytes([width[0], width[1], width[2], width[3]]),
                channel_count: *channel_count,
                color_space: *color_space,
            },
            &data[14..],
        ))
    }
}


// convert a color into a number from 0..64
pub fn color_hash(r: u8, g: u8, b: u8, a: u8) -> usize {
    (r as usize * 3 + g as usize * 5 + b as usize * 7 + a as usize * 11) % 64
}

#[inline]
pub fn add_hash_and_last(
    r: u8,
    g: u8,
    b: u8,
    a: u8,
    array: &mut [(u8, u8, u8, u8); 64],
    last: &mut (u8, u8, u8, u8),
) {
    let hash = color_hash(r, g, b, a);
    array[hash] = (r, g, b, a);
    *last = (r, g, b, a);
}

/// A decoded version of a qoi Part/Chunk
/// (I had to rewrite to add this API)
/// All values stored unbiased
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Part {
    RGBA(u8, u8, u8, u8),
    RGB(u8, u8, u8),
    Run(u8),
    SmallDiff(i8, i8, i8),
    /// drdg db dbdg
    LumaDiff(i8, i8, i8),
    Idx(u8),
}

impl Part {
    // TODO tests
    pub fn encode(&self, dst: &mut Vec<u8>) {
        match self {
            Part::RGBA(r,g,b,a) => {
                dst.push(0xff);
                dst.push(*r);
                dst.push(*g);
                dst.push(*b);
                dst.push(*a);
            }
            Part::RGB(r, g, b) => {
                dst.push(0xfe);
                dst.push(*r);
                dst.push(*g);
                dst.push(*b);
            }
            Part::SmallDiff(dr, dg, db) => {
                dst.push(0b0100_0000 | (((dr + 2) as u8) << 4) | (((dg + 2) as u8) << 2) | (db + 2) as u8)
            }
            Part::LumaDiff(drdg, dg, dbdg) => {
                dst.push(0b1000_0000 | (dg + 32) as u8);
                dst.push((((drdg+8) as u8) << 4) | (dbdg+8) as u8)
            }
            Part::Run(len) => {
                dst.push((len - 1) | 0b1100_0000)
            }
            Part::Idx(i) => {
                dst.push(*i)
            }
        }
    }
    // TODO unit tests
    pub fn decode<'a>(data: &'a [u8]) -> Option<(&'a [u8], Part)> {
        let first = *data.get(0)?;
        if first == 0b1111_1111 {
            // RGBA
            Some((
                data.get(5..)?,
                Part::RGBA(*data.get(1)?, *data.get(2)?, *data.get(3)?, *data.get(4)?)
            ))
        } else if first == 0b1111_1110 {
            // RGB
            Some((
                data.get(4..)?,
                Part::RGB(*data.get(1)?, *data.get(2)?, *data.get(3)?)
            ))
        } else if first & 0b1100_0000 == 0b00_00_0000 {
            // Index
            Some((
                data.get(1..)?,
                Part::Idx(first & 0b0011_1111)
            ))
        } else if first & 0b1100_0000 == 0b01_00_0000 {
            // Diff
            Some((
                data.get(1..)?,
                Part::SmallDiff(
                    ((first >> 4) & 0b11) as i8 - 2,
                    ((first >> 2) & 0b11) as i8 - 2, 
                    ((first >> 0) & 0b11) as i8 - 2
                )
            ))
        } else if first & 0b1100_0000 == 0b10_00_0000 {
            // Luma
            let a = data.get(0)?;
            let b = data.get(1)?;
            let dg = a & 0b11_1111;
            let dr = (b & 0b1111_0000) >> 4;
            let db = b & 0b0000_1111;
            Some((
                data.get(2..)?,
                Part::LumaDiff(dr as i8 - 8, dg as i8 - 32, db as i8 - 8)
            ))
        } else if first & 0b1100_0000 == 0b11_00_0000 {
            // Run
            Some((
                data.get(1..)?,
                Part::Run((first & 0b11_1111) + 1)
            ))
        } else {
            None
        }
    }
}

#[test]
fn decoding() {
    assert_eq!(Part::decode(&[0xff, 1,  2, 3, 4, 5]), Some((&[5][..], Part::RGBA(1,2,3,4))));
    assert_eq!(Part::decode(&[0xfe, 1,  2, 3, 4]), Some((&[4][..], Part::RGB(1,2,3))));
    assert_eq!(Part::decode(&[0b0000_1111, 4]), Some((&[4][..], Part::Idx(15))));
    assert_eq!(Part::decode(&[0b0100_0110, 4]), Some((&[4][..], Part::SmallDiff(-2, -1, 0))));
    assert_eq!(Part::decode(&[0b10_100000, 0b1000_0100, 4]), Some((&[4][..], Part::LumaDiff(0, 0, -4))));
    assert_eq!(Part::decode(&[0b1100_1111, 4]), Some((&[4][..], Part::Run(16))));
}

#[test]
fn encoding() {
    fn test(part: Part) {
        let mut v = vec![];
        part.encode(&mut v);
        v.push(0);
        println!("{:?}", v);
        assert_eq!(Part::decode(&v[..]), Some((&[0][..], part)));
    }
    test(Part::RGBA(0,1,2,2));
    test(Part::RGB(0,1,2));
    test(Part::LumaDiff(0,31,2));
    test(Part::SmallDiff(-1,1,0));
    test(Part::Idx(10));
    test(Part::Run(10));
}
