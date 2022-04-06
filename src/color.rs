pub fn bold(s: &str) -> String {
    color("1", s)
}

pub fn white(s: &str) -> String {
    s.to_string()
}

pub fn blue(s: &str) -> String {
    color("34", s)
}

pub fn green(s: &str) -> String {
    color("32", s)
}

pub fn red(s: &str) -> String {
    color("31", s)
}

pub fn yellow(s: &str) -> String {
    color("33", s)
}

fn color(code: &str, s: &str) -> String {
    format!("\x1b[{code}m{s}\x1b[0m")
}
