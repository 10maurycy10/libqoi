use crate::shared::*;

// (cpxl.0 as isize - last.0 as isize) >= -2 && (cpxl.0 as isize - last.0 as isize) <= 1 &&
//             (cpxl.1 as isize - last.1 as isize) >= -2 && (cpxl.1 as isize - last.1 as isize) <= 1 &&
//             (cpxl.2 as isize - last.2 as isize) >= -2 && (cpxl.2 as isize - last.2 as isize) <= 1 &&
//             (cpxl.3 == last.3 as isize)

//
// 299534 - RGBA + run of one
// 45730 - run len + RBG + RBGA
// 29689 - run len + index + RBG + RBGA
// 26098 - run len + small diff + index + RGB + RBGA
// 22356 - run len + small diff + diff luma + index + RGB + RBGA
// 21855 - run len + small wapping diff + diff wapping luma + index + RGB + RBGA

#[inline]
fn can_use_small_dif(a: (u8, u8, u8, u8), b: (u8, u8, u8, u8)) -> bool {
    let dr = (a.0 as i8).wrapping_sub(b.0 as i8);
    let dg = (a.1 as i8).wrapping_sub(b.1 as i8);
    let db = (a.2 as i8).wrapping_sub(b.2 as i8);
    let da = (a.3 as i8).wrapping_sub(b.3 as i8);

    if da != 0 {
        return false;
    }

    dr >= -2 && dr <= 1 && dg >= -2 && dg <= 1 && db >= -2 && db <= 1
}

#[inline]
fn can_use_luma(a: (u8, u8, u8, u8), b: (u8, u8, u8, u8)) -> bool {
    let dr = a.0 as isize - b.0 as isize;
    let dg = a.1 as isize - b.1 as isize;
    let db = a.2 as isize - b.2 as isize;
    let da = a.3 as isize - b.3 as isize;

    let drdg = dr - dg;
    let dbdg = db - dg;

    if da != 0 {
        return false;
    }

    dg >= -32 && dg <= 31 && drdg >= -8 && drdg <= 7 && dbdg >= -8 && dbdg <= 7
}

/// Compresses raw image data to QOI.
/// img_data :  Row major RGBA image data
/// hight, width: Image size
/// channel_count: 3 = RBG, 4 = RBGA
pub fn encode_qoi(
    img_data: &[u8],
    hight: usize,
    width: usize,
    channel_count: u8,
    color_space: u8,
) -> Option<Vec<u8>> {
    let mut buf = Vec::new();
    let header = Header {
        hight: hight as u32,
        width: width as u32,
        channel_count: channel_count,
        color_space: color_space,
    };
    encode_header(header, &mut buf);
    let mut colorhashes = [(0u8, 0u8, 0u8, 0u8); 64];
    let mut last = (0u8, 0u8, 0u8, 255u8);

    let mut rest = img_data;

    while rest.len() > 0 {
        let cpxl = (rest[0], rest[1], rest[2], rest[3]);

        // Run 1

        if cpxl.0 == last.0 && cpxl.1 == last.1 && cpxl.2 == last.2 && cpxl.3 == last.3 {
            // We have more than one pixel of the same color, encode as run
            let mut runlen = 1_usize;
            // TODO find best run len
            rest = &rest[4..];
            while rest[0] == cpxl.0 && rest[1] == cpxl.1 && rest[2] == cpxl.2 && rest[3] == cpxl.3 {
                runlen += 1;
                rest = &rest[4..];
                if rest.len() == 0 {
                    break;
                }
                // respect the max runlen
                if runlen == 62 {
                    break;
                }
            }
            Part::Run(runlen as u8).encode(&mut buf);
            // we dont need to update the hashes or last pxl for runs
            // TODO this is a bit slow (recomputes hashes)
        } else if colorhashes[color_hash(cpxl.0, cpxl.1, cpxl.2, cpxl.3)] == cpxl {
            Part::Idx(color_hash(cpxl.0, cpxl.1, cpxl.2, cpxl.3) as u8).encode(&mut buf);
            rest = &rest[4..];
            last = cpxl;
        // todo
        } else if can_use_small_dif(cpxl, last) {
            let dr = (cpxl.0 as i8).wrapping_sub(last.0 as i8);
            let dg = (cpxl.1 as i8).wrapping_sub(last.1 as i8);
            let db = (cpxl.2 as i8).wrapping_sub(last.2 as i8);
            //             println!("diff {} {} {}",dr, dg, db);
            Part::SmallDiff(dr as i8, dg as i8, db as i8).encode(&mut buf);
            rest = &rest[4..];
        } else if can_use_luma(cpxl, last) {
            let dr = (cpxl.0 as i8).wrapping_sub(last.0 as i8);
            let dg = (cpxl.1 as i8).wrapping_sub(last.1 as i8);
            let db = (cpxl.2 as i8).wrapping_sub(last.2 as i8);
            let drdg = dr.wrapping_sub(dg);
            let dbdg = db.wrapping_sub(dg);
            Part::LumaDiff(drdg as i8, dg as i8, dbdg as i8).encode(&mut buf);
            rest = &rest[4..];
        // if the last alpha is the same as the old one, use RGB
        } else if cpxl.3 == last.3 {
            let r = rest[0];
            let g = rest[1];
            let b = rest[2];
            // RGB encodes one pixel
            Part::RGB(r, g, b).encode(&mut buf);
            rest = &rest[4..];
        } else {
            // Otherwize fallback to RGBA
            let r = rest[0];
            let g = rest[1];
            let b = rest[2];
            let a = rest[3];
            //             println!("{:?}", (r,g,b,a));
            Part::RGBA(r, g, b, a).encode(&mut buf);
            // RGBA encodes one pixel
            rest = &rest[4..];
        }
        add_hash_and_last(cpxl.0, cpxl.1, cpxl.2, cpxl.3, &mut colorhashes, &mut last);
    }
    // Padding to help detect truncation
    buf.push(0x00);
    buf.push(0x00);
    buf.push(0x00);
    buf.push(0x00);
    buf.push(0x00);
    buf.push(0x00);
    buf.push(0x00);
    buf.push(0x01);

    Some(buf)
}
