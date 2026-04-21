use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize)]
pub struct ToGoRequest {
    pub json: String,
}

#[derive(Serialize)]
pub struct ToGoResponse {
    pub code: String,
}

pub async fn to_go(Json(req): Json<ToGoRequest>) -> Json<ToGoResponse> {
    let code = match serde_json::from_str::<Value>(&req.json) {
        Ok(value) => generate_go_struct(&value, "GeneratedStruct"),
        Err(e) => format!("// JSON 解析错误: {e}"),
    };
    Json(ToGoResponse { code })
}

fn to_go_field_name(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    if let Some(first) = result.chars().next() {
        if first.is_lowercase() {
            result = first.to_uppercase().to_string() + &result[1..];
        }
    }
    result
}

fn to_go_struct_name(s: &str) -> String {
    to_go_field_name(s)
}

fn infer_array_element_type(arr: &[Value], field_name: &str, nested_structs: &mut Vec<String>) -> String {
    if arr.is_empty() {
        return "[]interface{}".to_string();
    }
    match &arr[0] {
        Value::Null => "[]interface{}".to_string(),
        Value::Bool(_) => "[]bool".to_string(),
        Value::Number(n) => {
            if n.is_f64() && n.as_f64().map(|f| f.fract() != 0.0).unwrap_or(false) {
                "[]float64".to_string()
            } else {
                "[]int64".to_string()
            }
        }
        Value::String(_) => "[]string".to_string(),
        Value::Array(_) => "[][]interface{}".to_string(),
        Value::Object(_) => {
            let struct_name = to_go_struct_name(field_name);
            let nested_struct = generate_go_struct(&arr[0], &struct_name);
            nested_structs.push(nested_struct);
            format!("[]{struct_name}")
        }
    }
}

fn generate_go_struct(value: &Value, struct_name: &str) -> String {
    match value {
        Value::Object(obj) => {
            let mut fields = Vec::new();
            let mut nested_structs = Vec::new();
            for (key, val) in obj {
                let go_field_name = to_go_field_name(key);
                let go_struct_name = to_go_struct_name(key);
                let field_type = match val {
                    Value::Null => "interface{}".to_string(),
                    Value::Bool(_) => "bool".to_string(),
                    Value::Number(n) => {
                        if n.is_f64() && n.as_f64().map(|f| f.fract() != 0.0).unwrap_or(false) {
                            "float64".to_string()
                        } else {
                            "int64".to_string()
                        }
                    }
                    Value::String(_) => "string".to_string(),
                    Value::Array(arr) => infer_array_element_type(arr, key, &mut nested_structs),
                    Value::Object(_) => {
                        let nested_struct = generate_go_struct(val, &go_struct_name);
                        nested_structs.push(nested_struct);
                        go_struct_name.clone()
                    }
                };
                fields.push(format!("\t{go_field_name} {field_type} `json:\"{key}\"`"));
            }
            let nested_output = if nested_structs.is_empty() {
                String::new()
            } else {
                nested_structs.join("\n\n") + "\n\n"
            };
            format!(
                "{nested_output}type {struct_name} struct {{\n{}\n}}",
                fields.join("\n")
            )
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                format!("type {struct_name} []interface{{}}")
            } else if let Value::Object(_) = &arr[0] {
                let item_struct = generate_go_struct(&arr[0], &format!("{struct_name}Item"));
                format!("{item_struct}\n\ntype {struct_name} []{struct_name}Item")
            } else {
                format!("type {struct_name} []interface{{}}")
            }
        }
        _ => format!(
            "type {struct_name} struct {{\n\tValue interface{{}} `json:\"value\"`\n}}"
        ),
    }
}
