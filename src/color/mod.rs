pub const RESET: &str = "\x1b[0m";
pub const WHITE: &str = "\x1b[37m";
pub const BLUE: &str = "\x1b[34m";
pub const CYAN: &str = "\x1b[36m";
pub const RED: &str = "\x1b[31m";
pub const YELLOW: &str = "\x1b[33m";
pub const GREEN: &str = "\x1b[32m";
pub const BOLD: &str = "\x1b[1m";

pub fn bold(s: &str) -> String {
    color(BOLD, s)
}

pub fn white(s: &str) -> String {
    color(WHITE, s)
}

pub fn blue(s: &str) -> String {
    color(BLUE, s)
}

pub fn green(s: &str) -> String {
    color(GREEN, s)
}

pub fn red(s: &str) -> String {
    color(RED, s)
}

pub fn yellow(s: &str) -> String {
    color(YELLOW, s)
}

fn color(c: &str, s: &str) -> String {
    format!("{}{}{}", c, s, RESET)
}
