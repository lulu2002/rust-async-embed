use crate::app::button::ButtonDirection;

pub trait LedOperator {
    fn toggle(&mut self);
    fn shift(&mut self, direction: ButtonDirection);
}
