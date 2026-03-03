# Protocol V2: Stage-based Decomposition & Abstraction

## Overview

The current `Protocol` trait is a monolithic abstraction where each implementation handles the entire lifecycle of a request and response. To improve maintainability and support diverse protocols (OpenAI, Anthropic, Gemini, etc.), we will decompose the process into distinct, reusable stages.

## Protocol Lifecycle Stages

### 1. Schema Mapping (Request Assembly)
Convert the unified `ChatRequest` into a protocol-specific payload.

- **OpenAI Assembler**: Standard `content` field handling (String/Array).
- **Anthropic Assembler**: Moves `system` messages to a top-level field, requires `messages` to start with a user role.
- **Gemini Assembler**: Maps to `contents` array with `parts`.

### 2. Transport & Auth (Metadata Management)
Define endpoints, headers, and authentication strategies.

- **Bearer Auth**: Standard `Authorization: Bearer <key>`.
- **API Key Header**: `x-api-key: <key>`.
- **Signature V4**: AWS style complex signing (future).

### 3. Stream Interpretation (Real-time Parsing)
Transform raw bytes/SSE chunks into unified `ChatResponse` chunks.

- **SSE Interpreter**: Standard `data: ` line parsing.
- **Anthropic Streamer**: Handles specific event types (`message_start`, `content_block_delta`).
- **Raw JSON Streamer**: Handles direct JSON chunks (Ollama/Gemini).

### 4. Response Normalization (Post-processing)
Map the final JSON response back to `ChatResponse`.

- **OpenAI Mapper**: Standard `choices` indexing.
- **Anthropic Mapper**: Extracts from `content` array.

## Implementation Plan

### Phase 1: Directory Reorganization
Move from a flat `protocols/` structure to a component-based structure.

```text
src/protocols/
├── common/              # Shared logic (traits, auth, base types)
├── openai/              # OpenAI implementation
├── anthropic/           # Anthropic implementation
└── ...
```

### Phase 2: Trait Decomposition
Introduce granular traits if necessary, or at least structure the modules to expose these stages as decoupled functions.

### Phase 3: Migration
Migrate existing protocols (Aliyun, Zhipu, etc.) to use the `common` OpenAI components.
