use crate::{
    config::{ConfData, Config, Reader},
    isfs,
    mmio::vi::{VideoFormat, ViselDTV},
};

pub fn get_video_format() -> Option<VideoFormat> {
    let mut txt_buffer = isfs::read("/title/00000001/00000002/data/setting.txt").unwrap();
    Config::decrypt_txt_buf(&mut txt_buffer);
    let text = if let Err(err) = core::str::from_utf8(&txt_buffer) {
        unsafe { core::str::from_utf8_unchecked(&txt_buffer[..err.valid_up_to()]) }
    } else {
        return None;
    };

    for line in text.lines() {
        if let Some(char) = line.find("VIDEO=") {
            return match line[char + 6..].trim() {
                "NTSC" => Some(VideoFormat::Ntsc),
                "PAL" => Some(VideoFormat::Pal),
                "MPAL" => Some(VideoFormat::Mpal),
                _ => None,
            };
        }
    }
    None
}

pub fn has_component_cable() -> bool {
    ViselDTV::read().dtv() != 0
}

pub fn has_progressive_scan() -> bool {
    match Reader::new(isfs::read("/shared2/sys/SYSCONF").unwrap())
        .unwrap()
        .find("IPL.PGS")
        .unwrap()
        .1
    {
        ConfData::U8(data) => data != 0,
        _ => panic!(),
    }
}

pub fn has_eurgb60() -> bool {
    match Reader::new(isfs::read("/shared2/sys/SYSCONF").unwrap())
        .unwrap()
        .find("IPL.E60")
        .unwrap()
        .1
    {
        ConfData::U8(data) => data != 0,
        _ => panic!(),
    }
}

/*
pub fn get_preferred_video_mode() {
    let format = get_video_format().unwrap();
    if has_progressive_scan() && has_component_cable() {
        match format {
            VideoFormat::Ntsc => VideoMode::PROGRESSIVE_NTSC_480,
            VideoFormat::Pal => {
                if has_eurgb60() {
                    VideoMode::PRORESSIVE_EURGB60_480
                } else {
                    VideoMode::PROGRESSIVE_PAL_576
                }
            }
            VideoFormat::Mpal => VideoMode::PROGRESSIVE_MPAL_480,
            _ => VideoMode::PROGRESSIVE_NTSC_480,
        }
    } else {
        match format {
            VideoFormat::Ntsc => VideoMode::INTERLACED_NTSC_480,
            VideoFormat::Pal => {
                if has_eurgb60() {
                    VideoMode::INTERLACED_EURGB60_480
                } else {
                    VideoMode::INTERLACED_PAL_576
                }
            }
            VideoFormat::Mpal => VideoMode::INTERLACED_MPAL_480,
            _ => VideoMode::INTERLACED_NTSC_480,
        }
    }
    VideoMode::PROGRESSIVE_NTSC_480
}
*/

/*
pub fn set_adjusting_values(horizontal: usize, vertical: usize) {}

pub fn get_adjusting_values() -> (usize, usize) {}

pub fn get_next_framebuffer() -> *mut u8 {}

pub fn get_current_framebuffer() -> *mut u8 {}

pub fn init() {}

pub fn flush() {}

pub fn set_black(black_out: bool) {}

pub fn set_3d(3d: bool) {}

pub fn get_retrace_count() -> usize {}

pub fn get_next_field() -> RenderingField {}

pub fn get_current_line() -> usize {}

pub fn get_current_tv_mode() -> TvMode {}

pub fn get_scan_mode() -> ScanMode {}

pub fn configure(rendering_params: RenderingParams) {}

pub fn configure_pan(x_origin: usize, y_origin: usize, width: usize, height: usize) {}

pub fn get_framebuffer_size(rendering_params: RenderingParams) -> usize {}

pub fn clear_framebuffer(rendering_params: RenderingParams, framebuffer: *mut u8, color: YUYUV) {}

pub fn wait_vsync() {}

pub fn set_next_framebuffer(framebuffer: *mut u8) {}

pub fn set_next_right_framebuffer(framebuffer: *mut u8) {}


*/
