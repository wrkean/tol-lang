use std::{cell::RefCell, rc::Rc};

use crate::vm::{VM, list::List, range::Range, value::Value};

/// Built-in iterator
pub trait NativeIterator: std::fmt::Debug {
    /// Returns Some(Value) for the next item or None if exhausted
    fn next(&mut self, vm: &mut VM) -> Option<Value>;
}

#[derive(Debug)]
pub struct ListIterator {
    list: Rc<RefCell<List>>,
    index: usize,
}

impl ListIterator {
    pub fn new(list: Rc<RefCell<List>>) -> Self {
        Self { list, index: 0 }
    }
}

impl NativeIterator for ListIterator {
    fn next(&mut self, vm: &mut VM) -> Option<Value> {
        let val = self.list.borrow_mut().elements.get(self.index).cloned();
        self.index += 1;

        val
    }
}

#[derive(Debug)]
pub struct RangeIterator {
    current: i64,
    end: i64,
    step: i64,
    inclusive: bool,
    exhausted: bool,
}

impl RangeIterator {
    pub fn new(range: &Range) -> Self {
        Self {
            current: range.start,
            end: range.end,
            step: range.step,
            inclusive: range.inclusive,
            exhausted: false,
        }
    }
}

impl NativeIterator for RangeIterator {
    fn next(&mut self, vm: &mut VM) -> Option<Value> {
        if self.exhausted {
            return None;
        }

        let in_bounds = if self.step > 0 {
            if self.inclusive {
                self.current <= self.end
            } else {
                self.current < self.end
            }
        } else if self.step < 0 {
            if self.inclusive {
                self.current >= self.end
            } else {
                self.current > self.end
            }
        } else {
            vm.runtime_error("range step ay hindi dapat 0", vm.current_ip());
            return None;
        };

        if !in_bounds {
            self.exhausted = true;
            return None;
        }

        let value = self.current;
        self.current += self.step;

        if (self.step > 0 && self.current < value) || (self.step < 0 && self.current > value) {
            self.exhausted = true;
        }

        Some(Value::Int(value))
    }
}
