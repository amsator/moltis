# Vision & Philosophy

## What We're Building

Moltis is a **personal AI infrastructure controller** — not another chatbot UI.

```admonish info title="Core Principle"
One binary. Zero runtime dependencies. Full control over your AI stack.
```

## Blue Ocean vs Red Ocean

| Red Ocean (Don't Build) | Blue Ocean (Our Focus) |
|------------------------|------------------------|
| ChatGPT clone with web UI | CLI-agent integrated into terminal (zsh/fish) |
| "Smart" calendar/email | Infrastructure management agent (NixOS deploy, log monitoring) |
| RAG over Wikipedia | RAG over personal knowledge base (Obsidian, local PDF, code) |
| Universal agent | Micro-agent swarm (crypto watcher, server monitor, code reviewer) |

## Design Philosophy

### 1. Single Binary, Zero Runtime

```
No Node.js. No npm. No V8 garbage collector.
No "npm install" taking 5 minutes.
No dependency hell.
```

Moltis compiles everything — web UI, providers, tools, assets — into one executable. Start in milliseconds, not seconds.

### 2. Personal, Not Universal

We don't build for everyone. We build for:

- **Power users** who live in the terminal
- **Homelab operators** managing NixOS servers
- **Developers** who want AI integrated into their workflow
- **Privacy-conscious users** running everything locally

### 3. Infrastructure-First

Moltis is not a chatbot. It's an **AI operations platform**:

```
┌─────────────────────────────────────────────────────┐
│                    Moltis Core                      │
├─────────────────────────────────────────────────────┤
│  Server Monitor  │  Deploy Agent  │  Log Analyzer  │
│  Crypto Watcher  │  Code Reviewer │  Doc Writer    │
│  Calendar Bot    │  Email Triage  │  Backup Agent  │
└─────────────────────────────────────────────────────┘
                          │
                    Your Infrastructure
```

### 4. Swarm Architecture

One monolithic agent is fragile. A swarm of specialized micro-agents is resilient:

- **Server Agent**: Monitors logs, restarts services, alerts on anomalies
- **Deploy Agent**: Handles NixOS deployments, rollbacks, health checks
- **Knowledge Agent**: RAG over your Obsidian vault, PDFs, code
- **Comm Agent**: Telegram/Discord notifications, email triage
- **Crypto Agent**: Watch prices, alert on thresholds, execute strategies

Each agent has narrow scope, clear tools, and isolated failures.

### 5. Local-First Privacy

```
Your data stays on your machine.
Your secrets never leave your process.
Your conversations aren't logged to the cloud.
```

- OAuth tokens stored locally in `~/.config/moltis/`
- Embeddings computed locally (no external API calls)
- Optional cloud providers, never mandatory

## Target Use Cases

### DevOps / SRE

```bash
# Moltis watches your server logs and alerts on anomalies
moltis agent start --name log-watcher --prompt "Alert on 5xx spikes"

# Deploy NixOS configuration with rollback safety
moltis agent start --name deployer --prompt "Deploy to server, verify health, rollback on failure"
```

### Personal Knowledge Management

```bash
# RAG over your Obsidian vault
moltis memory sync ~/notes
moltis ask "What did I write about async Rust last month?"

# Summarize and file incoming PDFs
moltis agent start --name doc-filer --watch ~/Downloads/*.pdf
```

### Terminal Integration

```bash
# Integrated into zsh/fish
$ moltis shell
(moltis) $ explain this error: cargo build failed
(moltis) $ generate a nix module for this service
(moltis) $ review my last commit
```

### Micro-Services

```bash
# Spawn specialized agents
moltis swarm start \
  --agent crypto-watcher:watch BTC,ETH prices \
  --agent server-monitor:check logs every 5m \
  --agent backup-runner:backup every 6h
```

## What We're NOT Building

| We Don't Build | Why |
|----------------|-----|
| ChatGPT competitor | Market saturated, OpenAI wins |
| Enterprise SaaS | We're personal-first |
| Multi-tenant platform | Single-user, single-machine |
| No-code AI builder | We target developers |
| Browser-only interface | Terminal-first, browser-secondary |

## Success Metrics

1. **Startup time** < 100ms on commodity hardware
2. **Memory footprint** < 50MB idle (no loaded models)
3. **Binary size** < 100MB (including embedded UI)
4. **Zero runtime deps** — works on any Linux/macOS without installing Node/Python
5. **One-command install** — `curl | sh` or `brew install`

## The Road Ahead

### Phase 1: Foundation (Current)
- [x] Multi-provider support (Codex, Copilot, Local LLM)
- [x] Sandbox execution
- [x] Memory/RAG
- [x] Skills system
- [x] Hook system

### Phase 2: Swarm
- [ ] Multi-agent orchestration
- [ ] Agent-to-agent communication
- [ ] Specialized agent templates
- [ ] Background daemon mode

### Phase 3: Terminal Native
- [ ] zsh/fish integration
- [ ] Inline completions
- [ ] Pipe-through mode
- [ ] Interactive shell agent

### Phase 4: Infrastructure Control
- [ ] NixOS module generation
- [ ] Deployment automation
- [ ] Log monitoring
- [ ] Alert routing

---

```admonish tip title="Remember"
Moltis is for power users who want AI as infrastructure, not AI as a chat toy.
```
