// Text formatting
pub const BOLD: &str = "\x1B[1m";
pub const DIM: &str = "\x1B[2m";
pub const ITALIC: &str = "\x1B[3m";
pub const UNDERLINE: &str = "\x1B[4m";
pub const BLINK: &str = "\x1B[5m";
pub const REVERSE: &str = "\x1B[7m";
pub const HIDDEN: &str = "\x1B[8m";
pub const STRIKETHROUGH: &str = "\x1B[9m";

// Foreground colors
pub const BLACK: &str = "\x1B[30m";
pub const RED: &str = "\x1B[31m";
pub const GREEN: &str = "\x1B[32m";
pub const YELLOW: &str = "\x1B[33m";
pub const BLUE: &str = "\x1B[34m";
pub const MAGENTA: &str = "\x1B[35m";
pub const CYAN: &str = "\x1B[36m";
pub const WHITE: &str = "\x1B[37m";

pub const BRIGHT_BLACK: &str = "\x1B[90m";
pub const BRIGHT_RED: &str = "\x1B[91m";
pub const BRIGHT_GREEN: &str = "\x1B[92m";
pub const BRIGHT_YELLOW: &str = "\x1B[93m";
pub const BRIGHT_BLUE: &str = "\x1B[94m";
pub const BRIGHT_MAGENTA: &str = "\x1B[95m";
pub const BRIGHT_CYAN: &str = "\x1B[96m";
pub const BRIGHT_WHITE: &str = "\x1B[97m";

// Background colors
pub const BG_BLACK: &str = "\x1B[40m";
pub const BG_RED: &str = "\x1B[41m";
pub const BG_GREEN: &str = "\x1B[42m";
pub const BG_YELLOW: &str = "\x1B[43m";
pub const BG_BLUE: &str = "\x1B[44m";
pub const BG_MAGENTA: &str = "\x1B[45m";
pub const BG_CYAN: &str = "\x1B[46m";
pub const BG_WHITE: &str = "\x1B[47m";

pub const BG_BRIGHT_BLACK: &str = "\x1B[100m";
pub const BG_BRIGHT_RED: &str = "\x1B[101m";
pub const BG_BRIGHT_GREEN: &str = "\x1B[102m";
pub const BG_BRIGHT_YELLOW: &str = "\x1B[103m";
pub const BG_BRIGHT_BLUE: &str = "\x1B[104m";
pub const BG_BRIGHT_MAGENTA: &str = "\x1B[105m";
pub const BG_BRIGHT_CYAN: &str = "\x1B[106m";
pub const BG_BRIGHT_WHITE: &str = "\x1B[107m";

// Cursor movement
pub const CURSOR_UP: &str = "\x1B[A";
pub const CURSOR_DOWN: &str = "\x1B[B";
pub const CURSOR_RIGHT: &str = "\x1B[C";
pub const CURSOR_LEFT: &str = "\x1B[D";
pub const CURSOR_HOME: &str = "\x1B[H";
pub const CARRIAGE_RETURN: &str = "\r";

// Clearing
pub const CLEAR_LINE: &str = "\x1B[2K";
pub const CLEAR_LINE_START: &str = "\x1B[1K";
pub const CLEAR_LINE_END: &str = "\x1B[K";
pub const CLEAR_SCREEN: &str = "\x1B[2J";
pub const CLEAR_SCREEN_START: &str = "\x1B[1J";
pub const CLEAR_SCREEN_END: &str = "\x1B[J";
pub const CLEAR_SCROLLBACK: &str = "\x1B[3J";
