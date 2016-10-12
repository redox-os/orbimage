#![crate_name="orbimage"]
#![crate_type="lib"]

extern crate orbclient;
#[cfg(feature="png")]
extern crate png;

use std::cmp;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use orbclient::{Color, Window};

pub struct ImageRoi<'a> {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    image: &'a Image
}

impl<'a> ImageRoi<'a> {
    /// Draw the ROI on a window
    pub fn draw(&self, window: &mut Window, x: i32, mut y: i32) {
        let stride = self.image.w;
        let mut offset = (self.y * stride + self.x) as usize;
        let last_offset = cmp::min(((self.y + self.h) * stride + self.x) as usize, self.image.data.len());
        while offset < last_offset {
            let next_offset = offset + stride as usize;
            window.image(x, y, self.w, 1, &self.image.data[offset..]);
            offset = next_offset;
            y += 1;
        }
    }
}

pub struct Image {
    w: u32,
    h: u32,
    data: Box<[Color]>
}

impl Image {
    /// Create a new image
    pub fn new(width: u32, height: u32) -> Self {
        Self::from_color(width, height, Color::rgb(0, 0, 0))
    }

    /// Create a new image filled whole with color
    pub fn from_color(width: u32, height: u32, color: Color) -> Self {
        Self::from_data(width, height, vec![color; width as usize * height as usize].into_boxed_slice()).unwrap()
    }

    /// Create a new image from a boxed slice of colors
    pub fn from_data(width: u32, height: u32, data: Box<[Color]>) -> Result<Self, String> {
        if (width * height) as usize != data.len() {
            return Err("not enough or too much data given compared to width and height".to_string())
        }

        Ok(Image {
            w: width,
            h: height,
            data: data,
        })
    }

    /// Load an image from file path. Supports BMP and PNG
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let mut file = try!(File::open(&path).map_err(|err| format!("failed to open image: {}", err)));
        let mut data: Vec<u8> = Vec::new();
        let _ = try!(file.read_to_end(&mut data).map_err(|err| format!("failed to read image: {}", err)));
        //TODO: Use magic to match file instead of extension
        match path.as_ref().extension() {
            Some(extension_os) => match extension_os.to_str() {
                Some(extension) => match extension.to_lowercase().as_str() {
                    "bmp" => parse_bmp(&data),
                    "png" => parse_png(&data),
                    other => Err(format!("unknown image extension: {}", other))
                },
                None => Err("image extension not valid unicode".to_string())
            },
            None => Err("no image extension".to_string())
        }
    }

    /// Create a new empty image
    pub fn default() -> Self {
        Self::new(0, 0)
    }

    /// Get the width of the image in pixels
    pub fn width(&self) -> u32 {
        self.w
    }

    /// Get the height of the image in pixels
    pub fn height(&self) -> u32 {
        self.h
    }

    /// Get a piece of the image
    pub fn roi<'a>(&'a self, x: u32, y: u32, w: u32, h: u32) -> ImageRoi<'a> {
        let x1 = cmp::min(x, self.width());
        let y1 = cmp::min(y, self.height());
        let x2 = cmp::max(x1, cmp::min(x + w, self.width()));
        let y2 = cmp::max(y1, cmp::min(y + h, self.height()));

        ImageRoi {
            x: x1,
            y: y1,
            w: x2 - x1,
            h: y2 - y1,
            image: self
        }
    }

    /// Return a reference to a slice of colors making up the image
    pub fn data(&self) -> &[Color] {
        &self.data
    }

    /// Return a boxed slice of colors making up the image
    pub fn into_data(self) -> Box<[Color]> {
        self.data
    }

    /// Draw the image on a window
    pub fn draw(&self, window: &mut Window, x: i32, y: i32) {
        window.image(x, y, self.w, self.h, &self.data);
    }
}

#[cfg(feature="png")]
fn parse_png(file_data: &[u8]) -> Result<Image, String> {
    use png::ColorType::*;
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

#[cfg(not(feature="png"))]
fn parse_png(_file_data: &[u8]) -> Result<Image, String> {
    return Err("PNG support is not compiled in".to_string());
}

fn parse_bmp(file_data: &[u8]) -> Result<Image, String> {
    let get = |i: usize| -> u8 {
        match file_data.get(i) {
            Some(byte) => *byte,
            None => 0,
        }
    };

    let getw = |i: usize| -> u16 { (get(i) as u16) + ((get(i + 1) as u16) << 8) };

    let getd = |i: usize| -> u32 {
        (get(i) as u32) + ((get(i + 1) as u32) << 8) + ((get(i + 2) as u32) << 16) +
        ((get(i + 3) as u32) << 24)
    };

    let gets = |start: usize, len: usize| -> String {
        (start..start + len).map(|i| get(i) as char).collect::<String>()
    };

    if gets(0, 2) == "BM" {
        // let file_size = getd(2);
        let offset = getd(0xA);
        // let header_size = getd(0xE);
        let width = getd(0x12);
        let height = getd(0x16);
        let depth = getw(0x1C) as u32;

        let bytes = (depth + 7) / 8;
        let row_bytes = (depth * width + 31) / 32 * 4;

        let mut blue_mask = 0xFF;
        let mut green_mask = 0xFF00;
        let mut red_mask = 0xFF0000;
        let mut alpha_mask = 0xFF000000;
        if getd(0x1E) == 3 {
            red_mask = getd(0x36);
            green_mask = getd(0x3A);
            blue_mask = getd(0x3E);
            alpha_mask = getd(0x42);
        }

        let mut blue_shift = 0;
        while blue_mask > 0 && blue_shift < 32 && (blue_mask >> blue_shift) & 1 == 0 {
            blue_shift += 1;
        }

        let mut green_shift = 0;
        while green_mask > 0 && green_shift < 32 && (green_mask >> green_shift) & 1 == 0 {
            green_shift += 1;
        }

        let mut red_shift = 0;
        while red_mask > 0 && red_shift < 32 && (red_mask >> red_shift) & 1 == 0 {
            red_shift += 1;
        }

        let mut alpha_shift = 0;
        while alpha_mask > 0 && alpha_shift < 32 && (alpha_mask >> alpha_shift) & 1 == 0 {
            alpha_shift += 1;
        }

        let mut data = Vec::with_capacity(width as usize * height as usize);

        for y in 0..height {
            for x in 0..width {
                let pixel_offset = offset + (height - y - 1) * row_bytes + x * bytes;

                let pixel_data = getd(pixel_offset as usize);
                let red = ((pixel_data & red_mask) >> red_shift) as u8;
                let green = ((pixel_data & green_mask) >> green_shift) as u8;
                let blue = ((pixel_data & blue_mask) >> blue_shift) as u8;
                let alpha = ((pixel_data & alpha_mask) >> alpha_shift) as u8;
                if bytes == 3 {
                    data.push(Color::rgb(red, green, blue));
                } else if bytes == 4 {
                    data.push(Color::rgba(red, green, blue, alpha));
                }
            }
        }

        // This is not Ok(Image::from...) because Image started to return an Option
        // It shouldn't ever return an Err in this case, unless there's an error somewhere
        // above
        Image::from_data(width, height, data.into_boxed_slice())
    }else{
        Err("BMP: invalid signature".to_string())
    }
}
