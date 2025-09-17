# 🤖 AI Services API

*Comprehensive API reference for AI/ML model management and inference services*

## Overview

The AI Services API provides comprehensive endpoints for managing AI/ML models, running inference tasks, and monitoring AI service performance. All endpoints support both synchronous and asynchronous operations.

## Base URL

```
https://api.rust-ai-ide.dev/v1/ai
```

## Authentication

All AI endpoints require authentication:

```bash
Authorization: Bearer <jwt_token>
X-API-Key: <api_key>
```

## Model Management

### List Available Models

**GET** `/models`

Retrieve a list of all available AI models with their current status and capabilities.

```bash
curl -X GET "https://api.rust-ai-ide.dev/v1/ai/models" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json"
```

**Response:**
```json
{
  "models": [
    {
      "id": "codellama-7b",
      "name": "CodeLlama 7B",
      "type": "completion",
      "status": "loaded",
      "capabilities": ["code_completion", "refactoring"],
      "memory_usage_mb": 4096,
      "last_used": "2025-09-16T15:57:31Z",
      "performance_metrics": {
        "avg_inference_time_ms": 150,
        "success_rate": 0.98
      }
    },
    {
      "id": "starcoder-3b",
      "name": "StarCoder 3B",
      "type": "completion",
      "status": "available",
      "capabilities": ["code_generation", "documentation"],
      "memory_usage_mb": 2048,
      "performance_metrics": {
        "avg_inference_time_ms": 120,
        "success_rate": 0.97
      }
    }
  ],
  "pagination": {
    "total_count": 15,
    "has_more": false
  }
}
```

### Load Model

**POST** `/models/{model_id}/load`

Load a specific AI model into memory for inference.

```bash
curl -X POST "https://api.rust-ai-ide.dev/v1/ai/models/codellama-7b/load" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "priority": "high",
    "timeout_seconds": 300
  }'
```

**Request Body:**
```json
{
  "priority": "high|normal|low",
  "timeout_seconds": 300,
  "force_reload": false
}
```

**Response:**
```json
{
  "model_id": "codellama-7b",
  "status": "loading",
  "estimated_completion_time_seconds": 45,
  "queue_position": 0
}
```

### Unload Model

**POST** `/models/{model_id}/unload`

Unload a model from memory to free resources.

```bash
curl -X POST "https://api.rust-ai-ide.dev/v1/ai/models/codellama-7b/unload" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json"
```

**Response:**
```json
{
  "model_id": "codellama-7b",
  "status": "unloaded",
  "memory_freed_mb": 4096,
  "unload_time_ms": 150
}
```

## Inference Endpoints

### Code Completion

**POST** `/inference/completion`

Generate code completions using the specified model.

```bash
curl -X POST "https://api.rust-ai-ide.dev/v1/ai/inference/completion" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "model_id": "codellama-7b",
    "context": {
      "language": "rust",
      "code": "fn calculate_fibonacci(n: u32) -> u32 {\n    if n <= 1 {\n        return n;\n    }\n    // Complete this function",
      "cursor_position": 85
    },
    "parameters": {
      "max_tokens": 100,
      "temperature": 0.7,
      "top_p": 0.9
    }
  }'
```

**Request Body:**
```json
{
  "model_id": "codellama-7b",
  "context": {
    "language": "rust|python|typescript|go|java",
    "code": "string",
    "cursor_position": 85,
    "file_path": "src/main.rs",
    "project_context": {
      "dependencies": ["serde", "tokio"],
      "framework": "axum"
    }
  },
  "parameters": {
    "max_tokens": 100,
    "temperature": 0.1,
    "top_p": 0.9,
    "stop_sequences": ["\n\n", "```"],
    "frequency_penalty": 0.0,
    "presence_penalty": 0.0
  },
  "options": {
    "stream": true,
    "async": false,
    "cache": true
  }
}
```

**Response:**
```json
{
  "completion": {
    "text": "    calculate_fibonacci(n - 1) + calculate_fibonacci(n - 2)\n}",
    "confidence": 0.92,
    "metadata": {
      "model_used": "codellama-7b",
      "tokens_used": 45,
      "processing_time_ms": 150
    }
  },
  "suggestions": [
    {
      "text": "    match n {\n        0 => 0,\n        1 => 1,\n        _ => calculate_fibonacci(n - 1) + calculate_fibonacci(n - 2),\n    }",
      "confidence": 0.88,
      "reasoning": "More idiomatic Rust pattern matching"
    }
  ]
}
```

### Code Generation

**POST** `/inference/generation`

Generate new code based on natural language descriptions.

```bash
curl -X POST "https://api.rust-ai-ide.dev/v1/ai/inference/generation" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "model_id": "codellama-7b",
    "prompt": "Create a Rust function that implements a binary search tree with insert and search operations",
    "context": {
      "language": "rust",
      "existing_code": "use std::cmp::Ordering;\n\n// Add binary search tree implementation here"
    },
    "parameters": {
      "max_tokens": 500,
      "temperature": 0.3
    }
  }'
```

### Refactoring Suggestions

**POST** `/inference/refactor`

Get AI-powered refactoring suggestions for code improvement.

```bash
curl -X POST "https://api.rust-ai-ide.dev/v1/ai/inference/refactor" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "model_id": "codellama-7b",
    "code": "fn old_function(x: i32, y: i32) -> i32 { if x > y { x } else { y } }",
    "operation": "extract_variable",
    "context": {
      "language": "rust",
      "file_path": "src/main.rs"
    }
  }'
```

**Request Body:**
```json
{
  "model_id": "codellama-7b",
  "code": "string",
  "operation": "extract_variable|extract_method|rename|optimize_performance",
  "context": {
    "language": "rust",
    "file_path": "src/main.rs",
    "selection_start": 10,
    "selection_end": 50
  },
  "safety_checks": true
}
```

**Response:**
```json
{
  "refactoring": {
    "type": "extract_variable",
    "original_code": "fn old_function(x: i32, y: i32) -> i32 { if x > y { x } else { y } }",
    "refactored_code": "fn old_function(x: i32, y: i32) -> i32 {\n    let max_value = if x > y { x } else { y };\n    max_value\n}",
    "confidence": 0.95,
    "safety_score": 0.98,
    "explanation": "Extracted the conditional expression into a variable for better readability"
  },
  "alternatives": [
    {
      "type": "use_std_max",
      "code": "fn old_function(x: i32, y: i32) -> i32 {\n    std::cmp::max(x, y)\n}",
      "confidence": 0.87,
      "safety_score": 1.0
    }
  ]
}
```

## Model Orchestration

### Multi-Model Inference

**POST** `/orchestration/inference`

Run inference across multiple models for improved results.

```bash
curl -X POST "https://api.rust-ai-ide.dev/v1/ai/orchestration/inference" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "models": ["codellama-7b", "starcoder-3b"],
    "strategy": "majority_vote",
    "task": {
      "type": "completion",
      "context": {
        "language": "rust",
        "code": "fn calculate_sum(numbers: &[i32]) -> i32 {",
        "cursor_position": 35
      }
    }
  }'
```

**Request Body:**
```json
{
  "models": ["model_id_1", "model_id_2"],
  "strategy": "majority_vote|weighted_average|best_of_n|fallback",
  "weights": [0.7, 0.3],
  "task": {
    "type": "completion|generation|refactor",
    "context": {},
    "parameters": {}
  },
  "timeout_seconds": 60
}
```

**Response:**
```json
{
  "orchestration_result": {
    "strategy_used": "majority_vote",
    "model_results": [
      {
        "model_id": "codellama-7b",
        "result": "numbers.iter().sum()",
        "confidence": 0.91,
        "processing_time_ms": 120
      },
      {
        "model_id": "starcoder-3b",
        "result": "numbers.iter().sum::<i32>()",
        "confidence": 0.88,
        "processing_time_ms": 95
      }
    ],
    "final_result": "numbers.iter().sum()",
    "consensus_score": 0.89,
    "total_processing_time_ms": 140
  }
}
```

## Performance Monitoring

### Model Performance Metrics

**GET** `/monitoring/models/{model_id}/metrics`

Get detailed performance metrics for a specific model.

```bash
curl -X GET "https://api.rust-ai-ide.dev/v1/ai/monitoring/models/codellama-7b/metrics" \
  -H "Authorization: Bearer <token>"
```

**Response:**
```json
{
  "model_id": "codellama-7b",
  "metrics": {
    "inference_count": 15420,
    "average_inference_time_ms": 145,
    "p95_inference_time_ms": 280,
    "success_rate": 0.987,
    "memory_usage_mb": 4096,
    "gpu_utilization_percent": 78,
    "error_rate": 0.013,
    "cache_hit_rate": 0.65
  },
  "time_range": {
    "start": "2025-09-16T00:00:00Z",
    "end": "2025-09-16T15:57:31Z"
  }
}
```

### AI Service Health

**GET** `/health`

Check the health status of AI services.

```bash
curl -X GET "https://api.rust-ai-ide.dev/v1/ai/health" \
  -H "Authorization: Bearer <token>"
```

**Response:**
```json
{
  "status": "healthy",
  "services": {
    "model_registry": {
      "status": "healthy",
      "loaded_models": 5,
      "total_models": 15
    },
    "inference_engine": {
      "status": "healthy",
      "active_requests": 12,
      "queue_length": 3
    },
    "cache_service": {
      "status": "healthy",
      "hit_rate": 0.78,
      "memory_usage_mb": 512
    }
  },
  "system_resources": {
    "cpu_usage_percent": 45,
    "memory_usage_percent": 68,
    "gpu_memory_used_mb": 8192,
    "gpu_memory_total_mb": 24576
  }
}
```

## Batch Operations

### Batch Inference

**POST** `/batch/inference`

Process multiple inference requests in a single API call.

```bash
curl -X POST "https://api.rust-ai-ide.dev/v1/ai/batch/inference" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "requests": [
      {
        "id": "req_1",
        "model_id": "codellama-7b",
        "type": "completion",
        "context": {
          "language": "rust",
          "code": "fn main() {",
          "cursor_position": 12
        }
      },
      {
        "id": "req_2",
        "model_id": "starcoder-3b",
        "type": "generation",
        "prompt": "Create a Python function to validate email addresses"
      }
    ],
    "options": {
      "parallel_execution": true,
      "max_concurrent": 5,
      "timeout_seconds": 120
    }
  }'
```

**Response:**
```json
{
  "batch_id": "batch_abc123",
  "status": "completed",
  "results": [
    {
      "request_id": "req_1",
      "status": "success",
      "result": {
        "completion": {
          "text": "    println!(\"Hello, World!\");",
          "confidence": 0.94
        }
      },
      "processing_time_ms": 150
    },
    {
      "request_id": "req_2",
      "status": "success",
      "result": {
        "generation": {
          "text": "import re\n\ndef validate_email(email):\n    pattern = r'^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$'\n    return re.match(pattern, email) is not None",
          "confidence": 0.91
        }
      },
      "processing_time_ms": 200
    }
  ],
  "summary": {
    "total_requests": 2,
    "successful_requests": 2,
    "failed_requests": 0,
    "average_processing_time_ms": 175
  }
}
```

## Error Handling

### Common Error Responses

```json
{
  "error": {
    "code": "MODEL_NOT_FOUND",
    "message": "Requested model 'codellama-13b' is not available",
    "details": {
      "available_models": ["codellama-7b", "starcoder-3b"],
      "suggestion": "Use 'codellama-7b' instead"
    },
    "timestamp": "2025-09-16T15:57:31Z",
    "request_id": "req_xyz789"
  }
}
```

```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded for AI inference",
    "details": {
      "limit": 100,
      "window_seconds": 60,
      "retry_after_seconds": 30
    },
    "timestamp": "2025-09-16T15:57:31Z",
    "request_id": "req_xyz790"
  }
}
```

## Webhooks

### AI Event Webhooks

Configure webhooks to receive real-time notifications about AI service events.

**POST** `/webhooks/ai-events`

```json
{
  "url": "https://your-app.com/webhooks/ai-events",
  "events": ["model_loaded", "inference_completed", "error_occurred"],
  "secret": "webhook_secret_for_verification"
}
```

**Example Webhook Payload:**
```json
{
  "event_type": "inference_completed",
  "timestamp": "2025-09-16T15:57:31Z",
  "data": {
    "model_id": "codellama-7b",
    "request_id": "req_abc123",
    "processing_time_ms": 150,
    "tokens_used": 45,
    "result": {
      "confidence": 0.92,
      "text": "    calculate_fibonacci(n - 1) + calculate_fibonacci(n - 2)"
    }
  },
  "signature": "sha256_signature_for_verification"
}
```

## Rate Limits

- **Authenticated users**: 1000 requests per minute
- **AI inference**: 100 requests per minute
- **Model management**: 50 requests per minute
- **Batch operations**: 10 requests per minute

## Best Practices

1. **Model Selection**: Choose appropriate models based on task complexity and resource constraints
2. **Caching**: Enable caching for frequently used code patterns
3. **Batch Processing**: Use batch operations for multiple similar requests
4. **Error Handling**: Implement proper retry logic with exponential backoff
5. **Monitoring**: Monitor performance metrics and adjust parameters accordingly
6. **Resource Management**: Unload unused models to free memory
7. **Security**: Always validate input data and use HTTPS for all requests

## SDK Usage Examples

### JavaScript/TypeScript

```typescript
import { RustAIIDE } from '@rust-ai-ide/sdk';

const client = new RustAIIDE({
  apiKey: 'your-api-key',
  baseUrl: 'https://api.rust-ai-ide.dev/v1'
});

// Code completion
const completion = await client.ai.inference.completion({
  modelId: 'codellama-7b',
  context: {
    language: 'rust',
    code: 'fn main() {',
    cursorPosition: 12
  }
});

console.log(completion.text);
```

### Python

```python
from rust_ai_ide_sdk import RustAIIDE

client = RustAIIDE(api_key='your-api-key')

# Refactoring suggestion
result = client.ai.refactor({
    'model_id': 'codellama-7b',
    'code': 'fn old_func() { println!("hello"); }',
    'operation': 'extract_variable'
})

print(result['refactored_code'])
```

### Rust

```rust
use rust_ai_ide_sdk::RustAIIDE;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RustAIIDE::new("your-api-key")?;

    // Model management
    let models = client.ai.models().list().await?;
    println!("Available models: {:?}", models);

    // Inference
    let completion = client.ai.inference()
        .completion("codellama-7b", "fn main() {", 12)
        .await?;

    println!("Completion: {}", completion.text);

    Ok(())
}