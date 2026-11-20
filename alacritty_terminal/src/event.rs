use std::borrow::Cow;
use std::fmt::{self, Debug, Formatter};
use std::process::ExitStatus;
use std::sync::Arc;

use crate::grid::{Grid, Row};
use crate::term::ClipboardType;
use crate::term::cell::Cell;
use crate::vte::ansi::Rgb;

/// Terminal event.
///
/// These events instruct the UI over changes that can't be handled by the terminal emulation layer
/// itself.
#[derive(Clone)]
pub enum Event {
    /// Grid has changed possibly requiring a mouse cursor shape change.
    MouseCursorDirty,

    /// Window title change.
    Title(String),

    /// Reset to the default window title.
    ResetTitle,

    /// Request to store a text string in the clipboard.
    ClipboardStore(ClipboardType, String),

    /// Request to write the contents of the clipboard to the PTY.
    ///
    /// The attached function is a formatter which will correctly transform the clipboard content
    /// into the expected escape sequence format.
    ClipboardLoad(ClipboardType, Arc<dyn Fn(&str) -> String + Sync + Send + 'static>),

    /// Request to write the RGB value of a color to the PTY.
    ///
    /// The attached function is a formatter which will correctly transform the RGB color into the
    /// expected escape sequence format.
    ColorRequest(usize, Arc<dyn Fn(Rgb) -> String + Sync + Send + 'static>),

    /// Write some text to the PTY.
    PtyWrite(String),

    /// Request to write the text area size.
    TextAreaSizeRequest(Arc<dyn Fn(WindowSize) -> String + Sync + Send + 'static>),

    /// Cursor blinking state has changed.
    CursorBlinkingChange,

    /// New terminal content available.
    Wakeup,

    /// Terminal bell ring.
    Bell,

    /// Shutdown request.
    Exit,

    /// Child process exited.
    ChildExit(ExitStatus),
}

impl Debug for Event {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Event::ClipboardStore(ty, text) => write!(f, "ClipboardStore({ty:?}, {text})"),
            Event::ClipboardLoad(ty, _) => write!(f, "ClipboardLoad({ty:?})"),
            Event::TextAreaSizeRequest(_) => write!(f, "TextAreaSizeRequest"),
            Event::ColorRequest(index, _) => write!(f, "ColorRequest({index})"),
            Event::PtyWrite(text) => write!(f, "PtyWrite({text})"),
            Event::Title(title) => write!(f, "Title({title})"),
            Event::CursorBlinkingChange => write!(f, "CursorBlinkingChange"),
            Event::MouseCursorDirty => write!(f, "MouseCursorDirty"),
            Event::ResetTitle => write!(f, "ResetTitle"),
            Event::Wakeup => write!(f, "Wakeup"),
            Event::Bell => write!(f, "Bell"),
            Event::Exit => write!(f, "Exit"),
            Event::ChildExit(status) => write!(f, "ChildExit({status:?})"),
        }
    }
}

/// Byte sequences are sent to a `Notify` in response to some events.
pub trait Notify {
    /// Notify that an escape sequence should be written to the PTY.
    ///
    /// TODO this needs to be able to error somehow.
    fn notify<B: Into<Cow<'static, [u8]>>>(&self, _: B);
}

#[derive(Copy, Clone, Debug)]
pub struct WindowSize {
    pub num_lines: u16,
    pub num_cols: u16,
    pub cell_width: u16,
    pub cell_height: u16,
}

/// Types that are interested in when the display is resized.
pub trait OnResize {
    fn on_resize(&mut self, window_size: WindowSize);
}

/// Event Loop for notifying the renderer about terminal events.
pub trait EventListener {
    fn send_event(&self, _event: Event) {}

    /// A line is about to scroll off the top of the alternate screen.
    ///
    /// The alternate screen has no scrollback history, so this is the last time the line can be
    /// read; the line is borrowed only during the call and is cleared by the scroll after it. It
    /// is called for each line, from the top, when a line feed, `IND`, `NEL` or an automatic wrap
    /// of the input at the bottom margin, or `CSI S`, scrolls up a scroll region that starts at
    /// the top of the alternate screen. Deleting lines and resizing do not call it.
    ///
    /// Like the other methods of this trait it is called while the terminal handles input, so it
    /// must not block.
    fn alt_screen_line_leaving(&self, _line: &Row<Cell>) {}

    /// The terminal is about to switch from the primary screen to the alternate screen.
    ///
    /// It is called by [`Term::swap_alt`](crate::term::Term::swap_alt), which the input calls for
    /// `CSI ? 1049 h`, in order with [`EventListener::alt_screen_line_leaving`].
    fn alt_screen_entering(&self) {}

    /// The terminal is about to switch from the alternate screen to the primary screen.
    ///
    /// `screen` is the alternate screen as the program left it. It is called by
    /// [`Term::swap_alt`](crate::term::Term::swap_alt), which the input calls for `CSI ? 1049 l`,
    /// and by a full reset (`RIS`) on the alternate screen, before the reset clears the screen.
    fn alt_screen_leaving(&self, _screen: &Grid<Cell>) {}
}

/// Null sink for events.
pub struct VoidListener;

impl EventListener for VoidListener {}
