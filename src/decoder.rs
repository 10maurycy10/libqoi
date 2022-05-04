use crate::shared::*;

/// Decodes an QOI image to raw RGBA data row-major, returns metadata in a Header struct
pub fn decode_qoi<'a>(data: &'a [u8]) -> Option<(Header, Vec<u8>, &'a [u8])> {
    let (head, mut data) = read_header(data)?;
    let decoded_size = (head.hight * head.width * 4) as usize;
    let mut pxlbuffer = Vec::with_capacity(decoded_size);
    let mut colorhashes = [(0u8, 0u8, 0u8, 0u8); 64];
    let mut last_pxl = (0u8, 0u8, 0u8, 255u8);
    while pxlbuffer.len() < decoded_size {
        let (rest, part) = Part::decode(&data)?;
        data = rest;
        match part {
            Part::RGBA(r,g,b,a) => {
                pxlbuffer.push(r);
                pxlbuffer.push(g);
                pxlbuffer.push(b);
                pxlbuffer.push(a);
                add_hash_and_last(r, g, b, a, &mut colorhashes, &mut last_pxl);
            }
            Part::RGB(r,g,b) => {
                pxlbuffer.push(r);
                pxlbuffer.push(g);
                pxlbuffer.push(b);
                pxlbuffer.push(last_pxl.3);
                add_hash_and_last(r, g, b, last_pxl.3, &mut colorhashes, &mut last_pxl);
            }
            Part::Run(runlen) => {
                for _ in 0..runlen {
                    pxlbuffer.push(last_pxl.0);
                    pxlbuffer.push(last_pxl.1);
                    pxlbuffer.push(last_pxl.2);
                    pxlbuffer.push(last_pxl.3);
                }
            }
            Part::LumaDiff(drdg, dg, dbdg) => {
                let dr = drdg + dg;
                let db = dbdg + dg;
                let r = (last_pxl.0 as i8).wrapping_add(dr) as u8;
                let g = (last_pxl.1 as i8).wrapping_add(dg) as u8;
                let b = (last_pxl.2 as i8).wrapping_add(db) as u8;
                pxlbuffer.push(r);
                pxlbuffer.push(g);
                pxlbuffer.push(b);
                pxlbuffer.push(last_pxl.3);
                add_hash_and_last(r, g, b, last_pxl.3, &mut colorhashes, &mut last_pxl);
            }
            Part::SmallDiff(dr, dg, db) => {
                let r = (last_pxl.0 as i8).wrapping_add(dr) as u8;
                let g = (last_pxl.1 as i8).wrapping_add(dg) as u8;
                let b = (last_pxl.2 as i8).wrapping_add(db) as u8;
                pxlbuffer.push(r);
                pxlbuffer.push(g);
                pxlbuffer.push(b);
                pxlbuffer.push(last_pxl.3);
                add_hash_and_last(r, g, b, last_pxl.3, &mut colorhashes, &mut last_pxl);
            }
            Part::Idx(index) => {
                let pxl = colorhashes[index as usize];
                pxlbuffer.push(pxl.0);
                pxlbuffer.push(pxl.1);
                pxlbuffer.push(pxl.2);
                pxlbuffer.push(pxl.3);
                // Adding this to hash is redundent.
                add_hash_and_last(pxl.0, pxl.1, pxl.2, pxl.3, &mut colorhashes, &mut last_pxl);
            }
        }
    }
    Some((head, pxlbuffer, data))
}
