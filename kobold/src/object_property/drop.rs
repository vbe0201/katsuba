use super::Value;

pub fn safely(value: Value) {
    match value {
        Value::List(_) | Value::Object(_) => {}
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
            Value::Object(obj) => {
                for (_, child) in obj {
                    stack.push(child);
                }
            }
            _ => {}
        }
    }
}
