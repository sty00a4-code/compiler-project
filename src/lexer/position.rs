use std::fmt::{Debug, Display};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Position {
    pub ln: usize,
    pub col: usize,
}
impl Position {
    pub fn new(ln: usize, col: usize) -> Self {
        Self { ln, col }
    }
}
pub struct Located<T> {
    pub value: T,
    pub pos: Position,
}
impl<T> Located<T> {
    pub fn new(value: T, pos: Position) -> Self {
        Self { value, pos }
    }
    pub fn map<U, F: Fn(T) -> U>(self, f: F) -> Located<U> {
        Located { value: f(self.value), pos: self.pos }
    }
}
impl<T: Debug> Debug for Located<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl<T: Display> Display for Located<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.value.fmt(f)
    }
}
impl<T: Clone> Clone for Located<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
            pos: self.pos.clone(),
        }
    }
}
impl<T: PartialEq> PartialEq for Located<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl<T: Default> Default for Located<T> {
    fn default() -> Self {
        Self {
            value: T::default(),
            pos: Position::default(),
        }
    }
}
