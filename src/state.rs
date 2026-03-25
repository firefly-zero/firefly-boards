use crate::*;
use alloc::{
    str,
    string::{String, ToString},
    vec::Vec,
};
use core::cell::OnceCell;
use firefly_rust::*;
use firefly_types::Encode;
use firefly_ui::InputManager;

static mut STATE: OnceCell<State> = OnceCell::new();

pub struct Page {
    pub position: u16,
    pub name: String,
    pub scores: Vec<Score>,
}

pub struct Score {
    pub name: String,
    pub me: bool,
    pub value: i16,
    pub formatted: String,
}

pub struct State {
    pub settings: Settings,
    pub font: FileBuf,
    pub input: InputManager,
    pub pages: Option<Vec<Page>>,
    pub page: usize,
}

pub fn get_state() -> &'static mut State {
    #[allow(static_mut_refs)]
    unsafe { STATE.get_mut() }.unwrap()
}

pub fn load_state() {
    // Generally, you never want to use "get_me" peer
    // for anything but visual rendering. However,
    // the settings app is special. Settings must be
    // applied only on one device. The state drift is intentional.
    let peer = unsafe { Peer::from_u8(get_me().into_u8()) };

    let mut input = InputManager::new();
    input.peer = peer;

    let target = load_target();
    let pages = match &target {
        Some((author_id, app_id)) => load_pages(author_id, app_id),
        None => None,
    };

    if target.is_none() {
        log_error("failed to load target");
    } else if pages.is_none() {
        log_error("app has no badges");
    }
    if pages.is_none() {
        quit();
    }

    let state = State {
        settings: get_settings(peer),
        font: load_file_buf("ascii").unwrap(),
        pages,
        input,
        page: 0,
    };
    #[allow(static_mut_refs)]
    unsafe { STATE.set(state) }.ok().unwrap();
}

fn load_pages(author_id: &str, app_id: &str) -> Option<Vec<Page>> {
    let boards_path = alloc::format!("roms/{author_id}/{app_id}/_boards");
    let raw = sudo::load_file_buf(&boards_path)?;
    let boards = firefly_types::Boards::decode(raw.as_bytes()).ok()?;

    let stats_path = alloc::format!("data/{author_id}/{app_id}/stats");
    let raw = sudo::load_file_buf(&stats_path)?;
    let stats = firefly_types::Stats::decode(raw.as_bytes()).ok()?;

    if boards.boards.len() != stats.scores.len() {
        return None;
    }
    let mut pages = Vec::new();
    for (board, raw_scores) in boards.boards.iter().zip(stats.scores) {
        let scores = get_scores(board, raw_scores);
        if scores.is_empty() {
            continue;
        }
        let page = Page {
            position: board.position,
            name: board.name.to_string(),
            scores,
        };
        pages.push(page);
    }
    if pages.is_empty() {
        return None;
    }
    Some(pages)
}

fn get_scores(board: &firefly_types::Board, raw_scores: firefly_types::BoardScores) -> Vec<Score> {
    let mut scores = Vec::new();
    let peer = unsafe { Peer::from_u8(get_me().into_u8()) };
    let my_name = get_name_buf(peer);

    for score in raw_scores.me.iter() {
        let score = *score;
        if !valid_score(board, score) {
            continue;
        }
        scores.push(Score {
            name: my_name.clone(),
            formatted: format_score(board, score),
            value: score,
            me: true,
        });
    }
    let friend_names = load_friend_names();
    let default_name = "anonymous";
    for friend in raw_scores.friends.iter() {
        let score = friend.score;
        if !valid_score(board, score) {
            continue;
        }
        let friend_name = friend_names.get(usize::from(friend.index));
        let friend_name = match friend_name {
            Some(friend_name) => friend_name.clone(),
            None => default_name.to_string(),
        };
        scores.push(Score {
            name: friend_name.clone(),
            formatted: format_score(board, score),
            value: score,
            me: false,
        });
    }

    scores
}

fn load_friend_names() -> Vec<String> {
    let Some(raw) = sudo::load_file_buf("sys/friends") else {
        return Vec::new();
    };
    let mut raw = raw.into_vec();
    let mut raw = &mut raw[..];
    let mut names = Vec::new();
    while !raw.is_empty() {
        let size = usize::from(raw[0]);
        let name_raw = &raw[1..=size];
        let name = unsafe { str::from_utf8_unchecked(name_raw) };
        names.push(name.to_string());
        raw = &mut raw[size + 1..];
    }
    names
}

const fn valid_score(board: &firefly_types::Board, score: i16) -> bool {
    score != 0 && score >= board.min && score <= board.max
}

fn format_score(board: &firefly_types::Board, score: i16) -> String {
    let val = score.unsigned_abs();
    // TODO: format decimal time.
    if board.time {
        format_time(val)
    } else if board.decimals > 0 {
        format_decimal(val, board.decimals)
    } else {
        val.to_string()
    }
}

fn format_decimal(v: u16, prec: u8) -> String {
    let sep = (10u32).pow(u32::from(prec));
    let right = u32::from(v) % sep;
    let left = u32::from(v) / sep;
    alloc::format!("{left}.{right:00$}", usize::from(prec))
}

fn format_time(mut v: u16) -> String {
    let mut parts = Vec::new();
    while v > 0 {
        parts.push(alloc::format!("{:02}", v % 60));
        v /= 60;
    }
    reverse(&mut parts);
    parts.join(":")
}

fn reverse(parts: &mut [String]) {
    let size = parts.len();
    for i in 0..(size / 2) {
        parts.swap(i, size - i - 1);
    }
}

/// Read the ID of the app to be removed.
fn load_target() -> Option<(String, String)> {
    let raw = load_file_buf("target")?;
    let raw = raw.as_bytes();
    let raw = raw.trim_ascii();
    let raw = alloc::str::from_utf8(raw).ok()?;
    let (author, app) = split_by(raw, '.')?;
    let target = (String::from(author), String::from(&app[1..]));
    Some(target)
}

/// Split the string once at the given character.
fn split_by(input: &str, sep: char) -> Option<(&str, &str)> {
    let mut split_at = None;
    let sep: u8 = sep.try_into().unwrap();
    for (i, ch) in input.bytes().enumerate() {
        if ch == sep {
            split_at = Some(i);
            break;
        }
    }
    let split_at = split_at?;
    Some(input.split_at(split_at))
}
