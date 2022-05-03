use crate::*;
use crate::shared::*;

/// NOTE returns RGBA data even for an RGB imadge
pub fn decode_qoi<'a>(data: &'a [u8]) -> Option<(Header,Vec<u8>, &'a [u8])> {
    let (head, mut data) = read_header(data)?;
    let decoded_size = (head.hight * head.width * 4) as usize;
    let mut pxlbuffer = Vec::with_capacity(decoded_size);
//     println!("decoded size is {}", decoded_size);
    // "HashMap" of last seen pixel values.
    let mut colorhashes = [(0u8, 0u8, 0u8, 0u8);64];
    let mut last_pxl = (0u8,0u8,0u8,255u8);
//     println!("{},  {}", pxlbuffer.len(), decoded_size);
    while pxlbuffer.len() < decoded_size {
//         println!("remaing data len {}", data.len());
        if *data.get(0)? == 0b1111_1110 {
            // RBG
            let (r,g,b, rest) = decode_RGB(data)?;
            pxlbuffer.push(r);
            pxlbuffer.push(g);
            pxlbuffer.push(b);
            pxlbuffer.push(last_pxl.3); // the alpha is unchanged with RGB
//             println!("decoded RGB {:?}", (r,g,b));
            add_hash_and_last(r,g,b,last_pxl.3, &mut colorhashes, &mut last_pxl);
            data = rest;
        } else if *data.get(0)? == 0b1111_1111 {
            let (r,g,b, a, rest) = decode_RGBA(data)?;
            pxlbuffer.push(r);
            pxlbuffer.push(g);
            pxlbuffer.push(b);
            pxlbuffer.push(a);
//             println!("decoded RGBA {:?}", (r,g,b,a));
            add_hash_and_last(r,g,b,a, &mut colorhashes, &mut last_pxl);
            data = rest;
        } else if *data.get(0)? & 0b1100_0000== 0b1100_0000 {
            let (runlen, rest) = decode_run(data)?;
//             println!("decoded run of {}", runlen);
            // there is a bias in run len as a zero size run is redundant
            for _ in 0..(runlen.wrapping_add(1)) {
                pxlbuffer.push(last_pxl.0);
                pxlbuffer.push(last_pxl.1);
                pxlbuffer.push(last_pxl.2);
                pxlbuffer.push(last_pxl.3);
            }
            data = rest;
        } else if *data.get(0)? & 0b1100_0000== 0b1000_0000 {
            // DIF LUMA
            let (dg,dr_luma,db_luma,rest) =  decode_diffluma(data)?;
//             println!("dif luma {:?}", (dg, dr_luma, db_luma)); // -1, 0, 0
//             println!("{:?}", last_pxl);
            let dr = dr_luma + dg; // 
            let db = db_luma + dg; // 
            let r = (last_pxl.0 as i8).wrapping_add(dr) as u8;
            let g = (last_pxl.1 as i8).wrapping_add(dg) as u8;
            let b = (last_pxl.2 as i8).wrapping_add(db) as u8;
            pxlbuffer.push(r);
            pxlbuffer.push(g);
            pxlbuffer.push(b);
            pxlbuffer.push(last_pxl.3);
            add_hash_and_last(r, g, b, last_pxl.3, &mut colorhashes, &mut last_pxl);
            data = rest;
        } else if *data.get(0)? & 0b1100_0000== 0b0100_0000 {
            // DIF
            let (dr, dg, db, rest) = decode_diff(data)?;
//             println!("dif {:?}", (dr,dg,db));
            let r = (last_pxl.0 as i8).wrapping_add(dr) as u8;
            let g = (last_pxl.1 as i8).wrapping_add(dg) as u8;
            let b = (last_pxl.2 as i8).wrapping_add(db) as u8;
            pxlbuffer.push(r);
            pxlbuffer.push(g);
            pxlbuffer.push(b);
            pxlbuffer.push(last_pxl.3);
            add_hash_and_last(r, g, b,last_pxl.3, &mut colorhashes, &mut last_pxl);
            data = rest
        } else if *data.get(0)? & 0b1100_0000== 0b0000_0000 {
            // INDEX
            let (idx, rest) = decode_index(data)?;
//             println!("index {:?}", idx);
            let pxl = colorhashes[idx as usize];
            pxlbuffer.push(pxl.0);
            pxlbuffer.push(pxl.1);
            pxlbuffer.push(pxl.2);
            pxlbuffer.push(pxl.3);
            // Adding this to hash is redundent.
            add_hash_and_last(pxl.0,pxl.1,pxl.2,pxl.3, &mut colorhashes, &mut last_pxl);
            data = rest;
        } else {
            // Invalid data
//             println!("invalid part");
            return None
        //    break;
        };
//         println!("buffer is: {} {:?}", pxlbuffer.len(), last_pxl);
    }
//     print!("done");
    Some((head, pxlbuffer, data))
}
