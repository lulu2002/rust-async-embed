use crate::app::time::Timer;

pub enum LedState<'a> {
    Toggle,
    Wait(Timer<'a>),
}
