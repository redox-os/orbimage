#[cfg(feature="png")]
extern crate png;

use super::Image;

#[cfg(not(feature="png"))]
pub fn parse(_file_data: &[u8]) -> Result<Image, String> {
    return Err("PNG support is not compiled in".to_string());
}

#[cfg(feature="png")]
pub fn parse(file_data: &[u8]) -> Result<Image, String> {
    use orbclient::Color;
    use self::png::ColorType::*;

    let decoder = png::Decoder::new(file_data);
    let (info, mut reader) = try!(decoder.read_info().map_err(|err| format!("PNG read info error: {}", err)));
    let mut img_data = vec![0; info.buffer_size()];
    try!(reader.next_frame(&mut img_data).map_err(|err| format!("PNG read data error: {}", err)));

    let width = info.width;
    let height = info.height;

    let mut data = Vec::with_capacity(width as usize * height as usize);

    match info.color_type {
        RGB => {
            for rgb in img_data.chunks(3) {
                let r = rgb[0]; let g = rgb[1]; let b = rgb[2];
                data.push(Color::rgb(r, g, b));
            }
        },
        RGBA => {
            for rgba in img_data.chunks(4) {
                let r = rgba[0]; let g = rgba[1]; let b = rgba[2]; let a = rgba[3];
                data.push(Color::rgba(r, g, b, a));
            }
        },
        Grayscale => {
            for g in img_data {
                data.push(Color::rgb(g, g, g));
            }
        },
        GrayscaleAlpha => {
            for ga in img_data.chunks(2) {
                let g = ga[0]; let a = ga[1];
                data.push(Color::rgba(g, g, g, a));
            }
        },
        _ => return Err("Unknown PNG type".to_string())
    };

    // Not Ok(Image::from...) for same reason as below in parse_bmp.
    Image::from_data(width, height, data.into_boxed_slice())
}
