use libqoi::decode_qoi;
use std::io::Read;
use std::fs;

fn main() {
    let mut file: Vec<u8> = vec![];
    fs::File::open("kodim10.qoi").unwrap().read_to_end(&mut file).unwrap();
    let (header, img, _) = decode_qoi(&file).expect("Test File is invalid");
    println!("File is {} by {}", header.height, header.width);
    println!("Raw RGBA data is {} bytes", img.len());
}
