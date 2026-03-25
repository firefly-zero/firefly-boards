#![no_std]
#![no_main]
extern crate alloc;

mod state;

use firefly_rust::*;
use firefly_ui::Input;
use state::*;

#[unsafe(no_mangle)]
extern "C" fn boot() {
    load_state();
}

#[unsafe(no_mangle)]
extern "C" fn update() {
    let state = get_state();
    state.input.update();
    match state.input.get() {
        Input::Left => {
            if let Some(pages) = &state.pages {
                if state.page > 0 {
                    state.page -= 1;
                } else {
                    state.page = pages.len() - 1;
                }
            }
        }
        Input::Right | Input::Select => {
            if let Some(pages) = &state.pages {
                if state.page < pages.len() - 1 {
                    state.page += 1;
                } else {
                    state.page = 0;
                }
            }
        }
        Input::Back => quit(),
        _ => {}
    }
}

#[unsafe(no_mangle)]
extern "C" fn render() {
    let state = get_state();
    let theme = state.settings.theme;
    firefly_ui::draw_bg(theme);
    let Some(pages) = &state.pages else {
        return;
    };
    let font = state.font.as_font();
    let page = &pages[state.page];
    let pressed = state.input.pressed();
    firefly_ui::draw_title(&page.name, pressed, &font, theme.accent);
    draw_title_arrows(&theme, pressed);

    for (score, i) in page.scores.iter().zip(2..) {
        let color = if score.me {
            theme.accent
        } else {
            theme.primary
        };
        let mut point = Point::new(20, 12 + i * 13);
        draw_text(&score.name, &font, point, color);

        point.x = WIDTH - 20 - font.line_width_ascii(&score.formatted) as i32;
        draw_text(&score.formatted, &font, point, color);
    }
}

fn draw_title_arrows(theme: &Theme, pressed: bool) {
    const BOX_ML: i32 = 16;
    const BOX_MT: i32 = 16;
    const BOX_Y: i32 = BOX_MT;

    const CURSOR_ML: i32 = 4;
    const CURSOR_X: i32 = BOX_ML + CURSOR_ML;

    let style = Style::solid(theme.accent);
    let mut p = Point::new(CURSOR_X + 1, BOX_Y + 3);
    if pressed {
        p.x += 1;
        p.y += 1;
    }
    draw_triangle(
        Point::new(p.x, p.y + 4),
        Point::new(p.x + 4, p.y),
        Point::new(p.x + 4, p.y + 8),
        style,
    );

    p.x += WIDTH - 2 * CURSOR_X - 3;
    draw_triangle(
        Point::new(p.x, p.y + 4),
        Point::new(p.x - 4, p.y),
        Point::new(p.x - 4, p.y + 8),
        style,
    );
}
