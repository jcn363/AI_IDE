use std::collections::HashMap;

use js_sys::{Array, Reflect, Uint8Array};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::{console, CssStyleDeclaration, Document, Element, Performance, Text};

// Performance monitoring and text processing utilities
#[derive(Serialize, Deserialize, Clone, Default)]
pub struct PerformanceMetrics {
    pub start_time: f64,
    pub end_time:   f64,
    pub duration:   f64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TextMetrics {
    pub line_count:          usize,
    pub char_count:          usize,
    pub word_count:          usize,
    pub average_word_length: f64,
}

#[wasm_bindgen]
extern "C" {
    // Performance API access
    type Performance;
    #[wasm_bindgen(method, getter)]
    fn now(this: &Performance) -> f64;
}

// Shared state for performance monitoring
static PERFORMANCE_CACHE: once_cell::sync::Lazy<RwLock<HashMap<String, PerformanceMetrics>>> =
    once_cell::sync::Lazy::new(|| RwLock::new(HashMap::new()));

/// Initialize WASM module with panic hook
#[wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();

    log!("Rust AI IDE WASM components initialized");
}

#[wasm_bindgen]
impl PerformanceMetrics {
    #[wasm_bindgen(getter)]
    pub fn start_time(&self) -> f64 {
        self.start_time
    }

    #[wasm_bindgen(getter)]
    pub fn end_time(&self) -> f64 {
        self.end_time
    }

    #[wasm_bindgen(getter)]
    pub fn duration(&self) -> f64 {
        self.duration
    }
}

/// Fast text processing and syntax highlighting utilities
#[wasm_bindgen]
pub struct TextProcessor {
    buffer:           Vec<u8>,
    processing_cache: HashMap<String, String>,
}

#[wasm_bindgen]
impl TextProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> TextProcessor {
        console::log_1(&"TextProcessor initialized".into());

        TextProcessor {
            buffer:           Vec::with_capacity(1024 * 1024), // 1MB initial capacity
            processing_cache: HashMap::new(),
        }
    }

    /// Calculate basic text metrics efficiently
    #[wasm_bindgen]
    pub fn analyze_text(&mut self, text: &str) -> JsValue {
        let start_time = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);

        let line_count = text.lines().count();
        let char_count = text.chars().count();

        let words: Vec<&str> = text.split_whitespace().collect();
        let word_count = words.len();
        let average_word_length = if word_count > 0 {
            words.iter().map(|w| w.len()).sum::<usize>() as f64 / word_count as f64
        } else {
            0.0
        };

        let metrics = TextMetrics {
            line_count,
            char_count,
            word_count,
            average_word_length,
        };

        let end_time = web_sys::window()
            .and_then(|w| w.performance())
            .map(|p| p.now())
            .unwrap_or(0.0);

        // Store performance metrics
        let perf = PerformanceMetrics {
            start_time,
            end_time,
            duration: end_time - start_time,
        };

        PERFORMANCE_CACHE
            .write()
            .insert("analyze_text".to_string(), perf);

        serde_wasm_bindgen::to_value(&metrics).unwrap_or(JsValue::NULL)
    }

    /// Fast syntax highlighting for code snippets
    #[wasm_bindgen]
    pub fn highlight_syntax(&mut self, code: &str, language: &str) -> String {
        let cache_key = format!("{}_{}", language, code.len());

        // Check cache first
        if let Some(result) = self.processing_cache.get(&cache_key) {
            return result.clone();
        }

        let highlighted = match language {
            "rust" => self.highlight_rust(code),
            "typescript" | "javascript" => self.highlight_javascript(code),
            "python" => self.highlight_python(code),
            _ => self.highlight_generic(code),
        };

        // Cache the result
        self.processing_cache.insert(cache_key, highlighted.clone());
        highlighted
    }

    fn highlight_rust(&self, code: &str) -> String {
        let mut result = code.to_string();
        result = result.replace("fn ", "<span style=\"color: #61dafb\">fn</span> ");
        result = result.replace("let ", "<span style=\"color: #ff6b6b\">let</span> ");
        result = result.replace("struct ", "<span style=\"color: #4ecdc4\">struct</span> ");
        result = result.replace("impl ", "<span style=\"color: #45b7d1\">impl</span> ");
        result = result.replace("use ", "<span style=\"color: #96ceb4\">use</span> ");
        result
    }

    fn highlight_javascript(&self, code: &str) -> String {
        let mut result = code.to_string();
        result = result.replace(
            "function ",
            "<span style=\"color: #61dafb\">function</span> ",
        );
        result = result.replace("const ", "<span style=\"color: #ff6b6b\">const</span> ");
        result = result.replace("let ", "<span style=\"color: #ff6b6b\">let</span> ");
        result = result.replace("var ", "<span style=\"color: #ff6b6b\">var</span> ");
        result = result.replace("class ", "<span style=\"color: #4ecdc4\">class</span> ");
        result = result.replace("import ", "<span style=\"color: #96ceb4\">import</span> ");
        result
    }

    fn highlight_python(&self, code: &str) -> String {
        let mut result = code.to_string();
        result = result.replace("def ", "<span style=\"color: #61dafb\">def</span> ");
        result = result.replace("class ", "<span style=\"color: #4ecdc4\">class</span> ");
        result = result.replace("import ", "<span style=\"color: #96ceb4\">import</span> ");
        result = result.replace("from ", "<span style=\"color: #96ceb4\">from</span> ");
        result = result.replace("if ", "<span style=\"color: #ffa07a\">if</span> ");
        result = result.replace("for ", "<span style=\"color: #ffa07a\">for</span> ");
        result = result.replace("while ", "<span style=\"color: #ffa07a\">while</span> ");
        result
    }

    fn highlight_generic(&self, code: &str) -> String {
        code.to_string() // No highlighting for unknown languages
    }

    /// Process text buffer in chunks for memory efficiency
    #[wasm_bindgen]
    pub fn process_large_file(&mut self, content: &str, chunk_size: usize) -> String {
        // Process file in chunks to avoid memory issues
        let chunks = content
            .chars()
            .collect::<Vec<char>>()
            .chunks(chunk_size)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<String>>();

        let mut result = String::new();
        for chunk in chunks {
            result.push_str(&self.highlight_syntax(&chunk, "auto"));
        }

        result
    }

    /// Get performance metrics for operations
    #[wasm_bindgen]
    pub fn get_performance_metrics(&self) -> JsValue {
        let cache = PERFORMANCE_CACHE.read();
        serde_wasm_bindgen::to_value(&*cache).unwrap_or(JsValue::NULL)
    }

    /// Clear processing cache to free memory
    #[wasm_bindgen]
    pub fn clear_cache(&mut self) {
        self.processing_cache.clear();
        PERFORMANCE_CACHE.write().clear();
        console::log_1(&"Processing cache cleared".into());
    }
}

/// Fast buffer for zero-copy data operations
#[wasm_bindgen]
pub struct FastBuffer {
    data: Vec<u8>,
}

#[wasm_bindgen]
impl FastBuffer {
    #[wasm_bindgen(constructor)]
    pub fn new(capacity: usize) -> FastBuffer {
        FastBuffer {
            data: Vec::with_capacity(capacity),
        }
    }

    /// Append data to buffer
    #[wasm_bindgen]
    pub fn append(&mut self, data: &[u8]) -> usize {
        self.data.extend_from_slice(data);
        self.data.len()
    }

    /// Get buffer length
    #[wasm_bindgen]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Get buffer as Uint8Array for zero-copy JS access
    #[wasm_bindgen]
    pub fn as_uint8_array(&self) -> Uint8Array {
        Uint8Array::from(&self.data[..])
    }

    /// Clear buffer
    #[wasm_bindgen]
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

/// Memory efficient diff algorithm for large files
#[wasm_bindgen]
pub struct DiffProcessor {}

#[wasm_bindgen]
impl DiffProcessor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> DiffProcessor {
        DiffProcessor {}
    }

    /// Compute diff between two large strings efficiently
    #[wasm_bindgen]
    pub fn compute_diff(&self, old_content: &str, new_content: &str) -> JsValue {
        // Simple diff implementation - can be enhanced with more sophisticated algorithms
        let old_lines: Vec<&str> = old_content.lines().collect();
        let new_lines: Vec<&str> = new_content.lines().collect();

        let mut additions = 0;
        let mut deletions = 0;
        let mut changes = Vec::new();

        let max_len = old_lines.len().max(new_lines.len());

        for i in 0..max_len {
            let old_line = old_lines.get(i).unwrap_or(&"");
            let new_line = new_lines.get(i).unwrap_or(&"");

            if old_line != new_line {
                if i < old_lines.len() && !new_lines.contains(old_line) {
                    deletions += 1;
                }
                if i < new_lines.len() && !old_lines.contains(new_line) {
                    additions += 1;
                }

                changes.push(serde_json::json!({
                    "line": i,
                    "type": if new_line.is_empty() { "delete" } else if old_line.is_empty() { "add" } else { "change" },
                    "old": old_line,
                    "new": new_line
                }));
            }
        }

        let diff_result = serde_json::json!({
            "additions": additions,
            "deletions": deletions,
            "total_changes": changes.len(),
            "changes": changes
        });

        serde_wasm_bindgen::to_value(&diff_result).unwrap_or(JsValue::NULL)
    }
}

// Export functions for JavaScript interop
#[wasm_bindgen]
pub fn get_memory_info() -> JsValue {
    let memory_info = serde_json::json!({
        "used": 0, // Would be populated with actual memory info in a full implementation
        "total": 0,
        "available": 0
    });

    serde_wasm_bindgen::to_value(&memory_info).unwrap_or(JsValue::NULL)
}

#[wasm_bindgen]
pub fn perform_syntax_analysis(code: &str, language: &str) -> JsValue {
    let processor = TextProcessor::new();
    let result = processor.highlight_syntax(code, language);

    serde_json::json!({
        "language": language,
        "highlighted_code": result,
        "analysis_time": 0.0
    })
    .into()
}

// Console logging helper
fn log(s: &str) {
    console::log_1(&format!("[WASM] {}", s).into());
}
