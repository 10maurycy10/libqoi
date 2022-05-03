#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Header {
    /// big endian
    pub hight: u32,
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
    let h = head.hight.to_be_bytes();
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

pub fn read_header<'a>(data: &'a [u8]) -> Option<(Header, &'a [u8])> {
    let magic = data.get(0..4)?;
    let width = data.get(4..8)?;
    let hight = data.get(8..12)?;
    let channel_count = data.get(12)?;
    let color_space = data.get(13)?;
    if magic != b"qoif" {
        return None;
    } else {
        Some((
            Header {
                hight: u32::from_be_bytes([hight[0], hight[1], hight[2], hight[3]]),
                width: u32::from_be_bytes([width[0], width[1], width[2], width[3]]),
                channel_count: *channel_count,
                color_space: *color_space,
            },
            &data[14..],
        ))
    }
}

pub fn encode_rgb(r: u8, g: u8, b: u8, dst: &mut Vec<u8>) {
    dst.push(0xfe);
    dst.push(r);
    dst.push(g);
    dst.push(b);
}

pub fn encode_rgba(r: u8, g: u8, b: u8, a: u8, dst: &mut Vec<u8>) {
    dst.push(0xff);
    dst.push(r);
    dst.push(g);
    dst.push(b);
    dst.push(a);
}

// NOTICE WONT CHECK MAGIC!!
pub fn decode_rgb<'a>(data: &'a [u8]) -> Option<(u8, u8, u8, &'a [u8])> {
    Some((*data.get(1)?, *data.get(2)?, *data.get(3)?, data.get(4..)?))
}

// NOTICE WONT CHECK MAGIC!!
pub fn decode_rgba<'a>(data: &'a [u8]) -> Option<(u8, u8, u8, u8, &'a [u8])> {
    Some((
        *data.get(1)?,
        *data.get(2)?,
        *data.get(3)?,
        *data.get(4)?,
        data.get(5..)?,
    ))
}

pub fn encode_index(idx: u8, dst: &mut Vec<u8>) {
    dst.push(0b0011_1111 & idx);
}

// NOTE not biased
pub fn encode_run(idx: u8, dst: &mut Vec<u8>) {
    dst.push((0b0011_1111 & (idx)) | 0b1100_0000);
}

// NOTICE WONT CHECK MAGIC!!
pub fn decode_index<'a>(data: &'a [u8]) -> Option<(u8, &'a [u8])> {
    Some((*data.get(0)? & 0b0011_1111, data.get(1..)?))
}

// NOTICE WONT CHECK MAGIC!!
pub fn decode_run<'a>(data: &'a [u8]) -> Option<(u8, &'a [u8])> {
    Some(((*data.get(0)? & 0b0011_1111), data.get(1..)?))
}

pub fn encode_diff(dr: i8, dg: i8, db: i8, dst: &mut Vec<u8>) {
    let sr = (dr + 2) as u8 & 0b000_0011;
    let sg = (dg + 2) as u8 & 0b000_0011;
    let sb = (db + 2) as u8 & 0b000_0011;
    dst.push(0b0100_0000 | (sr << 4) | (sg << 2) | sb)
}

// NOTICE WONT CHECK MAGIC!!
pub fn decode_diff<'a>(data: &'a [u8]) -> Option<(i8, i8, i8, &'a [u8])> {
    let byte = data.get(0)?;
    let dr = (byte & 0b0011_0000) >> 4;
    let dg = (byte & 0b0000_1100) >> 2;
    let db = (byte & 0b0000_0011) >> 0;
    Some((dr as i8 - 2, dg as i8 - 2, db as i8 - 2, data.get(1..)?))
}

// NOTICE WONT CHECK MAGIC!!
pub fn decode_diffluma<'a>(data: &'a [u8]) -> Option<(i8, i8, i8, &'a [u8])> {
    let dg = data.get(0)? & 0b0011_1111;
    let drdg = (data.get(1)? & 0b1111_0000) >> 4;
    let dddg = data.get(1)? & 0b0000_1111;
    Some((
        dg as i8 - 32,
        drdg as i8 - 8,
        dddg as i8 - 8,
        data.get(2..)?,
    ))
}

pub fn encode_diffluma(drdg: i8, dg: i8, dbdg: i8, dst: &mut Vec<u8>) {
    dst.push(0b1000_0000 | (dg + 32) as u8);
    dst.push(((drdg + 8) << 4) as u8 | (dbdg + 8) as u8);
}

// convert a color into a number from 0..64
pub fn color_hash(r: u8, g: u8, b: u8, a: u8) -> usize {
    (r as usize * 3 + g as usize * 5 + b as usize * 7 + a as usize * 11) % 64
}

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
