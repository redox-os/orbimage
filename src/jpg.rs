#[cfg(feature="jpg")]
extern crate jpeg_decoder;

use super::Image;

#[cfg(not(feature="jpg"))]
pub fn parse(_file_data: &[u8]) -> Result<Image, String> {
    return Err("JPG support is not compiled in".to_string());
}

#[cfg(feature="jpg")]
pub fn parse(file_data: &[u8]) -> Result<Image, String> {
    use orbclient::Color;
    use self::jpeg_decoder::Decoder;
    use self::jpeg_decoder::PixelFormat::*;

    let mut decoder = Decoder::new(file_data);
    let img_data = decoder.decode().map_err(|err| format!("JPG read data error: {}", err))?;
    let info = decoder.info().ok_or(format!("JPG read info error"))?;

    let width = info.width;
    let height = info.height;

    let mut data = Vec::with_capacity(width as usize * height as usize);

    match info.pixel_format {
        L8 => {
            for g in img_data {
                data.push(Color::rgb(g, g, g));
            }
        },
        RGB24 => {
            for rgb in img_data.chunks(3) {
                let r = rgb[0]; let g = rgb[1]; let b = rgb[2];
                data.push(Color::rgb(r, g, b));
            }
        },
        CMYK32 => {
            for cmyk in img_data.chunks(4) {
                let c = cmyk[0] as f32 / 255.0;
                let m = cmyk[1] as f32 / 255.0;
                let y = cmyk[2] as f32 / 255.0;
                let k = cmyk[3] as f32 / 255.0;

                // CMYK -> CMY
                let c = c * (1.0 - k) + k;
                let m = m * (1.0 - k) + k;
                let y = y * (1.0 - k) + k;

                // CMY -> RGB
                let r = (1.0 - c) * 255.0;
                let g = (1.0 - m) * 255.0;
                let b = (1.0 - y) * 255.0;

                data.push(Color::rgb(r as u8, g as u8, b as u8));
            }
        }
    }

    Image::from_data(width as u32, height as u32, data.into_boxed_slice())
}
