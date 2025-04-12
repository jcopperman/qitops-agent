# qitops-agent

**The QitOps Agent** is an open-source AI-powered QA Assistant — built as the capstone project of [QitOps Learn](https://qitops.dev), and a flagship initiative of the QitOps testing philosophy.

It represents the culmination of a vision: to reimagine Quality Assurance not as an afterthought or gatekeeper, but as an **embedded, intelligent, human-centered force for stability, trust, and creativity in software development**.

---

## 🧠 What is the QitOps Agent?

The QitOps Agent is a modular, evolving AI assistant that helps testers, SDETs, and QA engineers:

- Plan smarter test strategies
- Generate meaningful test cases
- Analyze pull requests and surface risks
- Assist with exploratory and session-based testing
- Review flaky test histories and offer insights
- Support test data creation and edge-case scenarios
- Operate transparently, with explainability and human-in-the-loop confidence scores

This is more than just a tool — it’s a living expression of the QitOps methodology.

---

## 🌱 Project Goals

- 🎓 Serve as the **capstone project** of the QitOps Learn AI-Augmented QA track
- 🚀 Evolve as a **community-led open source tool** for AI-augmented software testing
- 📖 Demonstrate the **QitOps testing philosophy** in action: intelligent, adaptable, ethical QA by design
- 🛠 Build integrations with GitHub, Jira, Linear, and local dev/test environments
- 📦 Offer a **plugin-based architecture** to allow new agents, models, or extensions to be added modularly

---

## 🧩 Initial Feature Roadmap

### ✅ Phase 1: Foundational Agents
- [ ] Test Case Generator (Markdown / YAML / Robot Framework)
- [ ] Risk Estimator (PR / diff analysis)
- [ ] Bug Repro / Severity Analyzer
- [ ] Test Plan Suggestion Agent
- [ ] Test Data Generator

### 🔌 Phase 2: Integrations & Plugins
- [ ] GitHub PR integration
- [ ] CLI utility
- [ ] Local Ollama / OpenRouter LLM support
- [ ] p5.js-based session UI for exploratory feedback

### 🧪 Phase 3: Advanced Use Cases
- [ ] Historical flakiness detection
- [ ] QA knowledge synthesis from docs/repos
- [ ] Auto-generated QA dashboards
- [ ] Voice or chat-based QA coaching mode

---

## 🧠 Philosophy

This project is aligned with the values of QitOps:

- **Human-centered**: The agent supports testers, not replaces them
- **Intelligent**: Powered by LLMs, contextual memory, and explainability
- **Composable**: Works across disciplines, from manual to automation to exploratory
- **Transparent**: Outputs confidence scores, source insights, and prompts
- **Modular**: Built as a plugin system for rapid community experimentation

---

## 💡 Who Is This For?

- QA professionals curious about AI
- Developers who care about test quality
- Learners in the QitOps Learn program
- Open source contributors who want to shape the future of quality engineering

---

## 🚀 Installation & Usage

### Installation

```bash
# Clone the repository
git clone https://github.com/jcopperman/qitops-agent.git
cd qitops-agent

# Build the project
cargo build --release

# Run the binary
./target/release/qitops-agent --help
```

### Basic Commands

```bash
# Get help
qitops-agent --help

# Generate test cases
qitops-agent test-gen --path src/user/auth.rs --format markdown

# Analyze a pull request
qitops-agent pr-analyze --pr 123

# Estimate risk of changes
qitops-agent risk --diff changes.diff

# Generate test data
qitops-agent test-data --schema user-profile --count 100

# Start an interactive testing session
qitops-agent session --name "Login Flow Test"
```

### LLM Management

QitOps Agent supports multiple LLM providers:

```bash
# List available providers
qitops-agent llm list

# Add a new provider
qitops-agent llm add --provider openai --api-key YOUR_API_KEY --model gpt-4

# Set default provider
qitops-agent llm default --provider ollama

# Test a provider
qitops-agent llm test --provider anthropic --prompt "Generate a test case for user authentication"
```

## 🤝 Get Involved

Whether you're a prompt engineer, test automation expert, junior QA analyst, or just curious, you're welcome here.

- Browse the [good first issues](https://github.com/jcopperman/qitops-agent/issues)
- Share ideas in [QitOps Discord](#) *(coming soon)*
- Follow [@jcopperman](https://github.com/jcopperman) for updates

This project thrives on contribution, curiosity, and experimentation.

---

## 📜 License

MIT — Because quality should be open.

---

## 🧭 Inspired By

- The struggles and aspirations of testers everywhere
- The need for tools that *amplify* testers, not replace them
- A vision of QA as an operational, embedded force for good

Built with care, by [Jonathan](https://jcopperman.dev) — founder of QitOps, neurodivergent maker, and lifelong QA thinker.
