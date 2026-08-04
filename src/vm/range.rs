use std::fmt;

use crate::vm::value::Value;

#[derive(Debug, Clone)]
pub enum RangeNumber {
    Int(i64),
    Float(f64),
}

impl RangeNumber {
    pub fn from_value(value: &Value) -> Option<Self> {
        match value {
            Value::Int(int) => Some(Self::Int(*int)),
            Value::Float(float) => Some(Self::Float(*float)),
            _ => None,
        }
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Self::Float(_))
    }

    pub fn as_f64(&self) -> f64 {
        match self {
            Self::Int(int) => *int as f64,
            Self::Float(float) => *float,
        }
    }
}

impl fmt::Display for RangeNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Int(int) => write!(f, "{int}"),
            Self::Float(float) => write!(f, "{float}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Range {
    pub start: RangeNumber,
    pub end: RangeNumber,
    pub step: RangeNumber,
    pub inclusive: bool,
}

impl Range {
    pub fn from_values(start: &Value, end: &Value, step: &Value, inclusive: bool) -> Option<Self> {
        let start = RangeNumber::from_value(start)?;
        let end = RangeNumber::from_value(end)?;
        let step = RangeNumber::from_value(step)?;

        if start.is_float() || end.is_float() || step.is_float() {
            Some(Self {
                start: RangeNumber::Float(start.as_f64()),
                end: RangeNumber::Float(end.as_f64()),
                step: RangeNumber::Float(step.as_f64()),
                inclusive,
            })
        } else {
            Some(Self {
                start,
                end,
                step,
                inclusive,
            })
        }
    }

    pub fn with_step(&self, step: &Value) -> Option<Self> {
        let step = RangeNumber::from_value(step)?;
        if self.start.is_float() || self.end.is_float() || self.step.is_float() || step.is_float() {
            Some(Self {
                start: RangeNumber::Float(self.start.as_f64()),
                end: RangeNumber::Float(self.end.as_f64()),
                step: RangeNumber::Float(step.as_f64()),
                inclusive: self.inclusive,
            })
        } else {
            Some(Self {
                start: self.start.clone(),
                end: self.end.clone(),
                step,
                inclusive: self.inclusive,
            })
        }
    }
}
