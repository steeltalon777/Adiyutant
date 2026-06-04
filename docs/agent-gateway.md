# Agent Gateway

## Purpose

Define the boundary between AdiyutantCore and the AI agent layer. The Agent Gateway is an external component that handles LLM communication and tool execution.

## Current State

Agent Gateway is **not implemented** in MVP 0.1. No LLM integration exists.

## Planned Role

The Agent Gateway is responsible for:

- LLM provider routing (DeepSeek, Qwen, OpenAI, Gemini, Ollama, etc.)
- API key management and secure storage
- Tool execution and MCP-like scenarios
- Agent response generation
- Request filtering to minimize private context leakage

## Possible Implementations

| Type | Description | Status |
|------|-------------|--------|
| MockAgentGateway | Fixed responses for testing | Could be early MVP |
| LocalRuleAgentGateway | Simple rules without LLM | Could be early MVP |
| OpenClawAgentGateway | OpenClaw-based agent runtime | Future option |
| CustomServerAgentGateway | Custom Adiyutant server relay | Future option |

## Security Rule

- Core must not call LLM providers directly.
- Mobile client must not store provider API keys.
- API keys belong on the server-side or in the Agent Gateway.

## MVP 0.1 Decision

Agent Gateway is **out of scope** for MVP 0.1. Core will use LocalRuleAgentGateway (no LLM) for local suggestions.

## Open Questions

- Will OpenClaw be the first real Agent Gateway?
- Where will the gateway run? (local desktop relay, VPS, custom server)
- How will the gateway authenticate with the server?
- Will there be a local-only mode with mock gateway?
