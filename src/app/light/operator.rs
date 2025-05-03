use crate::app::btn::types::ButtonDirection;

pub trait LedOperator {
    fn toggle(&mut self);
    fn shift(&mut self, direction: ButtonDirection);
}
