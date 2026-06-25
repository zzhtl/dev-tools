use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use super::error::AppError;

// =====================================================================
// 格式互转：JSON ↔ YAML / TOML / XML / CSV，统一以 serde_json::Value 为中枢
// =====================================================================

#[derive(Deserialize)]
pub struct ConvertRequest {
    pub input: String,
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub indent: Option<usize>,
}

#[derive(Serialize)]
pub struct ConvertResponse {
    pub output: String,
}

pub async fn convert(Json(req): Json<ConvertRequest>) -> Result<Json<ConvertResponse>, AppError> {
    if req.input.trim().is_empty() {
        return Err(AppError::bad_request("输入为空"));
    }
    let value = parse_to_value(&req.input, &req.from)?;
    let output = serialize_value(&value, &req.to, req.indent.unwrap_or(2))?;
    Ok(Json(ConvertResponse { output }))
}

fn parse_to_value(input: &str, from: &str) -> Result<Value, AppError> {
    match from {
        "json" => serde_json::from_str(input)
            .map_err(|e| AppError::bad_request(format!("JSON 解析错误: {e}"))),
        "yaml" => serde_yaml::from_str(input)
            .map_err(|e| AppError::bad_request(format!("YAML 解析错误: {e}"))),
        "toml" => toml::from_str(input)
            .map_err(|e| AppError::bad_request(format!("TOML 解析错误: {e}"))),
        "csv" => csv_to_value(input),
        "xml" => xml_to_value(input),
        other => Err(AppError::bad_request(format!("不支持的源格式: {other}"))),
    }
}

fn serialize_value(value: &Value, to: &str, indent: usize) -> Result<String, AppError> {
    match to {
        "json" => to_json_pretty(value, indent),
        "yaml" => serde_yaml::to_string(value)
            .map_err(|e| AppError::bad_request(format!("YAML 序列化错误: {e}"))),
        "toml" => toml::to_string_pretty(value).map_err(|e| {
            AppError::bad_request(format!("TOML 序列化错误（要求顶层为对象且不含 null）: {e}"))
        }),
        "csv" => value_to_csv(value),
        "xml" => Ok(value_to_xml(value)),
        other => Err(AppError::bad_request(format!("不支持的目标格式: {other}"))),
    }
}

/// 以指定缩进美化 JSON。
fn to_json_pretty(value: &Value, indent: usize) -> Result<String, AppError> {
    let pad = " ".repeat(indent.clamp(0, 8));
    let mut buf = Vec::new();
    let formatter = serde_json::ser::PrettyFormatter::with_indent(pad.as_bytes());
    let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
    value
        .serialize(&mut ser)
        .map_err(|e| anyhow::anyhow!("JSON 序列化错误: {e}"))?;
    Ok(String::from_utf8(buf).map_err(|e| anyhow::anyhow!("{e}"))?)
}

// ---------- CSV ----------

/// CSV → JSON：首行为表头，每行映射为一个对象（值统一为字符串，CSV 本身无类型）。
fn csv_to_value(input: &str) -> Result<Value, AppError> {
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_reader(input.as_bytes());
    let headers = rdr
        .headers()
        .map_err(|e| AppError::bad_request(format!("CSV 表头解析错误: {e}")))?
        .clone();
    let mut rows = Vec::new();
    for record in rdr.records() {
        let record = record.map_err(|e| AppError::bad_request(format!("CSV 解析错误: {e}")))?;
        let mut map = serde_json::Map::new();
        for (i, field) in record.iter().enumerate() {
            let key = headers.get(i).unwrap_or("").to_string();
            map.insert(key, Value::String(field.to_string()));
        }
        rows.push(Value::Object(map));
    }
    Ok(Value::Array(rows))
}

/// JSON → CSV：要求顶层为对象数组，表头取所有对象键的并集（保留首次出现顺序）。
fn value_to_csv(value: &Value) -> Result<String, AppError> {
    let arr = value
        .as_array()
        .ok_or_else(|| AppError::bad_request("CSV 转换要求顶层为对象数组"))?;
    let mut headers: Vec<String> = Vec::new();
    for item in arr {
        let obj = item
            .as_object()
            .ok_or_else(|| AppError::bad_request("CSV 转换要求数组元素均为对象"))?;
        for k in obj.keys() {
            if !headers.iter().any(|h| h == k) {
                headers.push(k.clone());
            }
        }
    }
    let mut wtr = csv::WriterBuilder::new().from_writer(Vec::new());
    wtr.write_record(&headers)
        .map_err(|e| anyhow::anyhow!("CSV 写入错误: {e}"))?;
    for item in arr {
        let obj = item.as_object().unwrap();
        let row: Vec<String> = headers
            .iter()
            .map(|h| match obj.get(h) {
                None | Some(Value::Null) => String::new(),
                Some(Value::String(s)) => s.clone(),
                Some(v) => v.to_string(),
            })
            .collect();
        wtr.write_record(&row)
            .map_err(|e| anyhow::anyhow!("CSV 写入错误: {e}"))?;
    }
    let bytes = wtr
        .into_inner()
        .map_err(|e| anyhow::anyhow!("CSV 写入错误: {e}"))?;
    Ok(String::from_utf8(bytes).map_err(|e| anyhow::anyhow!("{e}"))?)
}

// ---------- XML（best-effort）----------

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// 反转 `xml_escape` 的 5 个标准实体（&amp; 最后处理，避免二次解码）。
fn xml_unescape(s: &str) -> String {
    s.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&amp;", "&")
}

/// JSON → XML：统一包一层 `<root>`；对象字段成为同名元素，数组成员重复父元素名。
fn value_to_xml(value: &Value) -> String {
    let mut out = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<root>\n");
    match value {
        Value::Object(map) => {
            for (k, v) in map {
                write_xml_element(&mut out, k, v, 1);
            }
        }
        Value::Array(arr) => {
            for item in arr {
                write_xml_element(&mut out, "item", item, 1);
            }
        }
        other => {
            out.push_str(&format!("  {}\n", xml_text(other)));
        }
    }
    out.push_str("</root>\n");
    out
}

fn write_xml_element(out: &mut String, name: &str, value: &Value, depth: usize) {
    let pad = "  ".repeat(depth);
    match value {
        Value::Object(map) => {
            out.push_str(&format!("{pad}<{name}>\n"));
            for (k, v) in map {
                write_xml_element(out, k, v, depth + 1);
            }
            out.push_str(&format!("{pad}</{name}>\n"));
        }
        Value::Array(arr) => {
            for item in arr {
                write_xml_element(out, name, item, depth);
            }
        }
        Value::Null => out.push_str(&format!("{pad}<{name}/>\n")),
        other => out.push_str(&format!("{pad}<{name}>{}</{name}>\n", xml_text(other))),
    }
}

fn xml_text(value: &Value) -> String {
    match value {
        Value::String(s) => xml_escape(s),
        other => other.to_string(),
    }
}

/// XML → JSON（best-effort）：纯文本元素 → 字符串；含子元素 → 对象；同名子元素 → 数组；
/// 属性以 `@name` 键保存；根元素名作为最外层键。
fn xml_to_value(input: &str) -> Result<Value, AppError> {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;

    let mut reader = Reader::from_str(input);
    reader.config_mut().trim_text(true);

    // 栈元素：(元素名, 子元素 map, 文本累加)
    let mut stack: Vec<(String, serde_json::Map<String, Value>, String)> = Vec::new();
    let mut root: Option<(String, Value)> = None;

    loop {
        match reader.read_event() {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                let mut map = serde_json::Map::new();
                push_attrs(&e, &mut map);
                stack.push((name, map, String::new()));
            }
            Ok(Event::Empty(e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).into_owned();
                let mut map = serde_json::Map::new();
                push_attrs(&e, &mut map);
                let value = xml_node_value(map, String::new());
                xml_insert(&mut stack, &mut root, &name, value);
            }
            Ok(Event::Text(e)) => {
                if let Some((_, _, text)) = stack.last_mut() {
                    let raw = e
                        .decode()
                        .map_err(|err| AppError::bad_request(format!("XML 解析错误: {err}")))?;
                    text.push_str(&xml_unescape(&raw));
                }
            }
            Ok(Event::End(_)) => {
                if let Some((name, map, text)) = stack.pop() {
                    let value = xml_node_value(map, text);
                    xml_insert(&mut stack, &mut root, &name, value);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(AppError::bad_request(format!("XML 解析错误: {e}"))),
            _ => {}
        }
    }

    match root {
        Some((name, value)) => Ok(json!({ name: value })),
        None => Err(AppError::bad_request("XML 为空或无根元素")),
    }
}

fn push_attrs(e: &quick_xml::events::BytesStart, map: &mut serde_json::Map<String, Value>) {
    for attr in e.attributes().flatten() {
        let key = format!("@{}", String::from_utf8_lossy(attr.key.as_ref()));
        let val = String::from_utf8_lossy(&attr.value).into_owned();
        map.insert(key, Value::String(val));
    }
}

fn xml_node_value(map: serde_json::Map<String, Value>, text: String) -> Value {
    let text = text.trim().to_string();
    if map.is_empty() {
        Value::String(text)
    } else {
        let mut map = map;
        if !text.is_empty() {
            map.insert("#text".to_string(), Value::String(text));
        }
        Value::Object(map)
    }
}

fn xml_insert(
    stack: &mut [(String, serde_json::Map<String, Value>, String)],
    root: &mut Option<(String, Value)>,
    name: &str,
    value: Value,
) {
    if let Some((_, parent_map, _)) = stack.last_mut() {
        match parent_map.get_mut(name) {
            Some(Value::Array(arr)) => arr.push(value),
            Some(existing) => {
                let prev = existing.take();
                *existing = Value::Array(vec![prev, value]);
            }
            None => {
                parent_map.insert(name.to_string(), value);
            }
        }
    } else {
        *root = Some((name.to_string(), value));
    }
}

// =====================================================================
// JSON Schema：校验 / 生成
// =====================================================================

#[derive(Deserialize)]
pub struct SchemaRequest {
    pub json: String,
    pub mode: String,
    #[serde(default)]
    pub schema: Option<String>,
}

#[derive(Serialize)]
pub struct SchemaErrorItem {
    pub path: String,
    pub message: String,
}

#[derive(Serialize, Default)]
pub struct SchemaResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Vec<SchemaErrorItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema: Option<String>,
}

pub async fn schema(Json(req): Json<SchemaRequest>) -> Result<Json<SchemaResponse>, AppError> {
    let instance: Value = serde_json::from_str(&req.json)
        .map_err(|e| AppError::bad_request(format!("JSON 解析错误: {e}")))?;
    match req.mode.as_str() {
        "generate" => {
            let schema = generate_schema(&instance);
            Ok(Json(SchemaResponse {
                schema: Some(to_json_pretty(&schema, 2)?),
                ..Default::default()
            }))
        }
        "validate" => {
            let schema_str = req
                .schema
                .ok_or_else(|| AppError::bad_request("缺少 schema 字段"))?;
            let schema_value: Value = serde_json::from_str(&schema_str)
                .map_err(|e| AppError::bad_request(format!("Schema JSON 解析错误: {e}")))?;
            let validator = jsonschema::validator_for(&schema_value)
                .map_err(|e| AppError::bad_request(format!("Schema 无效: {e}")))?;
            let errors: Vec<SchemaErrorItem> = validator
                .iter_errors(&instance)
                .map(|e| {
                    let path = e.instance_path().to_string();
                    SchemaErrorItem {
                        path: if path.is_empty() { "(根)".to_string() } else { path },
                        message: e.to_string(),
                    }
                })
                .collect();
            Ok(Json(SchemaResponse {
                valid: Some(errors.is_empty()),
                errors: Some(errors),
                ..Default::default()
            }))
        }
        other => Err(AppError::bad_request(format!("不支持的模式: {other}"))),
    }
}

/// 从样例 JSON 反推 draft-07 Schema。
fn generate_schema(value: &Value) -> Value {
    let inferred = infer_schema(value);
    if let Value::Object(map) = &inferred {
        let mut ordered = serde_json::Map::new();
        ordered.insert(
            "$schema".to_string(),
            Value::String("http://json-schema.org/draft-07/schema#".to_string()),
        );
        for (k, v) in map {
            ordered.insert(k.clone(), v.clone());
        }
        Value::Object(ordered)
    } else {
        inferred
    }
}

fn infer_schema(value: &Value) -> Value {
    match value {
        Value::Null => json!({ "type": "null" }),
        Value::Bool(_) => json!({ "type": "boolean" }),
        Value::Number(n) => {
            if n.is_f64() {
                json!({ "type": "number" })
            } else {
                json!({ "type": "integer" })
            }
        }
        Value::String(_) => json!({ "type": "string" }),
        Value::Array(arr) => {
            let items = arr.first().map(infer_schema).unwrap_or_else(|| json!({}));
            json!({ "type": "array", "items": items })
        }
        Value::Object(map) => {
            let mut props = serde_json::Map::new();
            let mut required = Vec::new();
            for (k, v) in map {
                props.insert(k.clone(), infer_schema(v));
                required.push(Value::String(k.clone()));
            }
            json!({ "type": "object", "properties": props, "required": required })
        }
    }
}

// =====================================================================
// 查询：JSONPath（serde_json_path）/ jq（jaq）
// =====================================================================

#[derive(Deserialize)]
pub struct QueryRequest {
    pub json: String,
    pub engine: String,
    pub expr: String,
}

#[derive(Serialize)]
pub struct QueryResponse {
    pub result: String,
}

pub async fn query(Json(req): Json<QueryRequest>) -> Result<Json<QueryResponse>, AppError> {
    if req.expr.trim().is_empty() {
        return Err(AppError::bad_request("查询表达式为空"));
    }
    let result = match req.engine.as_str() {
        "jsonpath" => run_jsonpath(&req.json, &req.expr)?,
        "jq" => run_jq(&req.json, &req.expr)?,
        other => return Err(AppError::bad_request(format!("不支持的查询引擎: {other}"))),
    };
    Ok(Json(QueryResponse { result }))
}

fn run_jsonpath(json: &str, expr: &str) -> Result<String, AppError> {
    let value: Value = serde_json::from_str(json)
        .map_err(|e| AppError::bad_request(format!("JSON 解析错误: {e}")))?;
    let path = serde_json_path::JsonPath::parse(expr)
        .map_err(|e| AppError::bad_request(format!("JSONPath 解析错误: {e}")))?;
    let nodes: Vec<Value> = path.query(&value).all().into_iter().cloned().collect();
    to_json_pretty(&Value::Array(nodes), 2)
}

fn run_jq(json: &str, expr: &str) -> Result<String, AppError> {
    use jaq_core::load::{Arena, File, Loader};
    use jaq_core::{data, unwrap_valr, Compiler, Ctx, Vars};
    use jaq_json::{read, Val};

    let input = read::parse_single(&json.as_bytes())
        .map_err(|e| AppError::bad_request(format!("JSON 解析错误: {e:?}")))?;

    let program = File {
        code: expr,
        path: (),
    };
    let defs = jaq_core::defs().chain(jaq_std::defs()).chain(jaq_json::defs());
    let funs = jaq_core::funs().chain(jaq_std::funs()).chain(jaq_json::funs());

    let loader = Loader::new(defs);
    let arena = Arena::default();
    let modules = loader
        .load(&arena, program)
        .map_err(|errs| AppError::bad_request(format!("jq 表达式解析失败: {errs:?}")))?;
    let filter = Compiler::default()
        .with_funs(funs)
        .compile(modules)
        .map_err(|errs| AppError::bad_request(format!("jq 表达式编译失败: {errs:?}")))?;

    let ctx = Ctx::<data::JustLut<Val>>::new(&filter.lut, Vars::new([]));
    let outputs = filter.id.run((ctx, input)).map(unwrap_valr);

    let mut lines = Vec::new();
    for out in outputs {
        match out {
            Ok(val) => lines.push(val.to_string()),
            Err(e) => return Err(AppError::bad_request(format!("jq 执行错误: {e:?}"))),
        }
    }
    Ok(lines.join("\n"))
}
