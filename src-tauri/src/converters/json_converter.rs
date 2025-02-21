use serde_json::Value;

#[tauri::command]
pub fn json_to_go_struct(json: &str) -> String {
    let parsed: Value = match serde_json::from_str(json) {
        Ok(value) => value,
        Err(_) => return String::from("Invalid JSON"),
    };

    fn generate_go_struct(value: &Value, struct_name: &str) -> String {
        match value {
            Value::Object(obj) => {
                let mut fields = Vec::new();
                let mut nested_structs = Vec::new();
                for (key, val) in obj {
                    let field_type = match val {
                        Value::Null => "interface{}".to_string(),
                        Value::Bool(_) => "bool".to_string(),
                        Value::Number(n) => {
                            if n.is_f64() {
                                "float64".to_string()
                            } else {
                                "int64".to_string()
                            }
                        }
                        Value::String(_) => "string".to_string(),
                        Value::Array(arr) => {
                            if arr.is_empty() {
                                "[]interface{}".to_string()
                            } else {
                                match &arr[0] {
                                    Value::Object(_) => {
                                        let nested_struct = generate_go_struct(&arr[0], key);
                                        nested_structs.push(nested_struct);
                                        format!("[]{}", key)
                                    }
                                    _ => "[]interface{}".to_string(),
                                }
                            }
                        }
                        Value::Object(_) => {
                            let nested_struct = generate_go_struct(val, key);
                            nested_structs.push(nested_struct);
                            key.to_string()
                        }
                    };
                    fields.push(format!("    {} {}", key, field_type));
                }
                format!(
                    "{}\ntype {} struct {{\n{}\n}}",
                    nested_structs.join("\n"),
                    struct_name,
                    fields.join("\n")
                )
            }
            _ => format!(
                "type {} struct {{\n    Value interface{{}}\n}}",
                struct_name
            ),
        }
    }

    generate_go_struct(&parsed, "GeneratedStruct")
}