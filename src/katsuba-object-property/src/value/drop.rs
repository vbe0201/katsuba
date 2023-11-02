use super::Value;

/// Safely drops `value` in heap memory.
///
/// This avoids stack overflows with deeply nested types.
pub fn safely(value: Value) {
    match value {
        Value::List(..) | Value::Object { .. } => {}
        _ => return,
    }

    let mut stack = Vec::new();
    stack.push(value);
    while let Some(value) = stack.pop() {
        match value {
            Value::List(list) => {
                for child in list {
                    stack.push(child);
                }
            }
            Value::Object { hash: _, obj } => {
                for (_, child) in obj {
                    stack.push(child);
                }
            }
            _ => (),
        }
    }
}
