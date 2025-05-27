pub mod diagnostic;
pub mod localizable;
pub mod types;

#[allow(dead_code)]
pub mod symbol_table;

pub fn format_float(val: f64) -> String {
    if val.fract() == 0.0 {
        format!("{:.1}", val)
    } else {
        format!("{}", val)
    }
}
