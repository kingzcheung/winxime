use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
use winxime_config::XimeConfig;

pub fn hex_to_rgb(hex: u32) -> (f32, f32, f32) {
    let r = ((hex >> 16) & 0xFF) as f32 / 255.0;
    let g = ((hex >> 8) & 0xFF) as f32 / 255.0;
    let b = (hex & 0xFF) as f32 / 255.0;
    (r, g, b)
}

pub fn get_colors(
    primary_color: u32,
) -> (
    D2D1_COLOR_F,
    D2D1_COLOR_F,
    D2D1_COLOR_F,
    D2D1_COLOR_F,
    D2D1_COLOR_F,
    D2D1_COLOR_F,
    D2D1_COLOR_F,
) {
    let primary = hex_to_rgb(primary_color);

    let bg_color = D2D1_COLOR_F {
        r: 0.95,
        g: 0.95,
        b: 0.95,
        a: 0.85,
    };
    let border_color = D2D1_COLOR_F {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.05,
    };
    let fg_color = D2D1_COLOR_F {
        r: 0.2,
        g: 0.2,
        b: 0.2,
        a: 1.0,
    };
    let selkey_color = D2D1_COLOR_F {
        r: 0.4,
        g: 0.4,
        b: 0.4,
        a: 1.0,
    };
    let comment_color = D2D1_COLOR_F {
        r: 0.5,
        g: 0.5,
        b: 0.5,
        a: 0.7,
    };
    let highlight_bg_color = D2D1_COLOR_F {
        r: primary.0,
        g: primary.1,
        b: primary.2,
        a: 0.9,
    };
    let highlight_fg_color = D2D1_COLOR_F {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };

    (
        bg_color,
        border_color,
        fg_color,
        selkey_color,
        comment_color,
        highlight_bg_color,
        highlight_fg_color,
    )
}
