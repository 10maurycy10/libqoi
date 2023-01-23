use libqoi::encode_qoi;
use std::io::Read;
use std::fs;

fn main() {
    let (h, w) = (512, 512);
    // Generate some image data to encode
    let mut img: Vec<u8> = vec![0; h*w*4];
    // Do it.
    let file = encode_qoi(&img, 512, 512, 4, 0).unwrap();
    println!("Encoded {} pixels into {} bytes", img.len()/4, file.len())
}
