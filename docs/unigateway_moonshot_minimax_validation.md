# UniGateway Integration with Moonshot / MiniMax Validation Guide

Purpose: Quickly identify whether issues are on the vendor side, llm-connector side, or in UniGateway calling methods.

## Validation Scope

- Direct vendor connection (control group)
- Forwarded via `unigateway`
- Compare key fields: `id`, `model`, `choices[0].message.content`, `usage`

## Script List

- `scripts/verify_direct_vendors.sh`
  - Direct requests to Moonshot and MiniMax official APIs.
- `scripts/verify_via_unigateway.sh`
  - Use same request body via gateway calls.

## Prerequisites

- `bash`
- `curl`
- `jq` (optional, recommended for formatted output)

## 1) Direct Vendor Validation (Baseline)

Set environment variables first:

```bash
export MOONSHOT_API_KEY='your moonshot key'
export MINIMAX_API_KEY='your minimax key'
# Optional
export MOONSHOT_MODEL='kimi-k2.5'
export MINIMAX_MODEL='MiniMax-M2.5'
```

Execute:

```bash
bash scripts/verify_direct_vendors.sh
```

Expected:

- Both vendors return HTTP 200
- `id/model/content/usage` are not empty

## 2) Validation via UniGateway

Set gateway information:

```bash
export GATEWAY_BASE_URL='http://<gateway-host>:<port>/v1'
export GATEWAY_API_KEY='your gateway key'
```

### 2.1 Moonshot Path

```bash
export TARGET_VENDOR='moonshot'
# Optional (adjust according to your gateway routing rules)
export GATEWAY_MODEL='kimi-k2.5'
bash scripts/verify_via_unigateway.sh
```

### 2.2 MiniMax Path

```bash
export TARGET_VENDOR='minimax'
# Optional (adjust according to your gateway routing rules)
export GATEWAY_MODEL='MiniMax-M2.5'
bash scripts/verify_via_unigateway.sh
```

Expected:

- Consistent with direct connection results, `id/model/content/usage` should not be cleared.

## 3) How to Determine "Gateway Called Incorrectly"

If direct connection works normally but gateway shows following response, you can preliminarily determine there's an issue with gateway forwarding/mapping:

```json
{"choices":[{"finish_reason":"stop","index":0,"message":{"content":"","role":"assistant"}}],"created":0,"id":"","model":"","object":"chat.completion","usage":null}
```

## 4) Common Error Points (for UniGateway Troubleshooting)

- MiniMax endpoint incorrect:
  - Some scenarios use `/v1/text/chatcompletion_v2`, not standard `/chat/completions`.
- MiniMax parameters incorrect:
  - Whether `max_tokens` needs to be mapped to `max_completion_tokens`.
- Gateway response mapping losing fields:
  - Whether `choices[].message.content`, `id`, `model`, `usage` are overwritten with default values.
- Streaming format parsed as OpenAI SSE:
  - If upstream is non-standard chunk, parsing with OpenAI format will lose content.

## 5) Recommended Materials to Provide to UniGateway

- Original output of both scripts (direct + gateway)
- Original response body when failed (before secondary mapping)
- Gateway request logs (upstream URL, request body, response body)
