/// Parsed inbound request (from --json mode stdin).
pub struct Request {
    pub command: String,
    pub source: Option<String>,
    pub file_path: Option<String>,
}

impl Request {
    pub fn parse(input: &str) -> Result<Self, String> {
        let v = json::parse(input).map_err(|e| format!("JSON parse error: {e}"))?;
        let command = v["command"]
            .as_str()
            .ok_or("missing 'command' field")?
            .to_string();
        let source = v["source"].as_str().map(|s| s.to_string());
        let file_path = v["file_path"].as_str().map(|s| s.to_string());
        Ok(Self { command, source, file_path })
    }
}

// ---------------------------------------------------------------------------
// Response builder
// ---------------------------------------------------------------------------

pub struct Response {
    pub ok: bool,
    pub command: String,
    pub formatted: Option<String>,
    pub changed: bool,
    pub timing_ms: u128,
    pub diagnostics: Vec<Diagnostic>,
    pub error: Option<String>,
}

impl Response {
    pub fn error(msg: String) -> Self {
        Self {
            ok: false,
            command: String::new(),
            formatted: None,
            changed: false,
            timing_ms: 0,
            diagnostics: vec![],
            error: Some(msg),
        }
    }

    pub fn to_json(&self) -> String {
        let diags: Vec<json::JsonValue> = self
            .diagnostics
            .iter()
            .map(|d| d.to_json_value())
            .collect();

        let mut obj = json::object! {
            ok: self.ok,
            command: self.command.as_str(),
            changed: self.changed,
            timing_ms: self.timing_ms as f64,
            diagnostics: json::JsonValue::Array(diags),
        };

        if let Some(f) = &self.formatted {
            obj["formatted"] = f.as_str().into();
        }
        if let Some(e) = &self.error {
            obj["error"] = e.as_str().into();
        }

        obj.dump()
    }
}

// ---------------------------------------------------------------------------

pub struct Diagnostic {
    pub line: usize,
    pub col: usize,
    pub end_line: usize,
    pub end_col: usize,
    pub message: String,
    pub severity: String,
    pub code: String,
    pub fix: Option<DiagnosticFix>,
}

impl Diagnostic {
    fn to_json_value(&self) -> json::JsonValue {
        let mut obj = json::object! {
            line: self.line,
            col: self.col,
            end_line: self.end_line,
            end_col: self.end_col,
            message: self.message.as_str(),
            severity: self.severity.as_str(),
            code: self.code.as_str(),
        };
        if let Some(fix) = &self.fix {
            obj["fix"] = json::object! {
                replacement: fix.replacement.as_str(),
            };
        }
        obj
    }
}

pub struct DiagnosticFix {
    pub replacement: String,
}
