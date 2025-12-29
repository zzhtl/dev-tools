use serde_json::Value;

/// 将字符串转换为 Go 语言的导出字段名格式（首字母大写的驼峰命名）
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
    
    // 确保第一个字符大写（Go导出字段必须首字母大写）
    if let Some(first) = result.chars().next() {
        if first.is_lowercase() {
            result = first.to_uppercase().to_string() + &result[1..];
        }
    }
    
    result
}

/// 将字符串转换为 Go 结构体名格式
fn to_go_struct_name(s: &str) -> String {
    to_go_field_name(s)
}

/// 根据 JSON 值推断 Go 数组元素类型
fn infer_array_element_type(arr: &[Value], field_name: &str, nested_structs: &mut Vec<String>) -> String {
    if arr.is_empty() {
        return "[]interface{}".to_string();
    }
    
    // 检查数组第一个元素的类型
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
            format!("[]{}", struct_name)
        }
    }
}

/// 递归生成 Go 结构体
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
                        // 检查是否为浮点数
                        if n.is_f64() && n.as_f64().map(|f| f.fract() != 0.0).unwrap_or(false) {
                            "float64".to_string()
                        } else {
                            "int64".to_string()
                        }
                    }
                    Value::String(_) => "string".to_string(),
                    Value::Array(arr) => {
                        infer_array_element_type(arr, key, &mut nested_structs)
                    }
                    Value::Object(_) => {
                        let nested_struct = generate_go_struct(val, &go_struct_name);
                        nested_structs.push(nested_struct);
                        go_struct_name.clone()
                    }
                };
                
                // 添加 JSON tag 以保持与原始 JSON 字段名的映射
                fields.push(format!("\t{} {} `json:\"{}\"`", go_field_name, field_type, key));
            }
            
            let nested_output = if nested_structs.is_empty() {
                String::new()
            } else {
                nested_structs.join("\n\n") + "\n\n"
            };
            
            format!(
                "{}type {} struct {{\n{}\n}}",
                nested_output,
                struct_name,
                fields.join("\n")
            )
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                format!("type {} []interface{{}}", struct_name)
            } else if let Value::Object(_) = &arr[0] {
                // 数组中的对象，生成结构体
                let item_struct = generate_go_struct(&arr[0], &format!("{}Item", struct_name));
                format!("{}\n\ntype {} []{}Item", item_struct, struct_name, struct_name)
            } else {
                format!("type {} []interface{{}}", struct_name)
            }
        }
        _ => format!(
            "type {} struct {{\n\tValue interface{{}} `json:\"value\"`\n}}",
            struct_name
        ),
    }
}

#[tauri::command]
pub fn json_to_go_struct(json: &str) -> String {
    let parsed: Value = match serde_json::from_str(json) {
        Ok(value) => value,
        Err(e) => return format!("// JSON 解析错误: {}", e),
    };

    generate_go_struct(&parsed, "GeneratedStruct")
}