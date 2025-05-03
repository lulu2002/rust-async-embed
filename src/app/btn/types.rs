use crate::app::time::Timer;

#[derive(Copy, Clone)]
pub enum ButtonDirection {
    Left,
    Right,
}

pub enum ButtonState<'a> {
    WaitForPress,
    Debounce(Timer<'a>),
    WaitForRelease,
}
