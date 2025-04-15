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
- [x] Test Case Generator (Markdown / YAML / Robot Framework)
- [x] Risk Estimator (PR / diff analysis)
- [x] Bug Repro / Severity Analyzer
- [x] Test Plan Suggestion Agent
- [x] Test Data Generator

### 🔌 Phase 2: Integrations & Plugins
- [x] GitHub PR integration
- [x] CLI utility
- [x] Local Ollama / OpenRouter LLM support
- [x] Interactive chat interface for QitOps Bot
- [x] Interactive testing sessions
- [x] Monitoring and metrics collection

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

### Prerequisites

- **Rust**: QitOps Agent is built with Rust. [Install Rust](https://www.rust-lang.org/tools/install) if you don't have it already.
- **Git**: Required to clone the repository.
- **LLM Provider**: You'll need access to an LLM provider:
  - **Ollama** (recommended for local use): [Install Ollama](https://ollama.ai/download)
  - **OpenAI API Key** (optional): For using OpenAI models
  - **Anthropic API Key** (optional): For using Claude models

### Installation

#### From Source (Recommended)

##### Windows

```powershell
# Clone the repository
git clone https://github.com/jcopperman/qitops-agent.git
cd qitops-agent

# Build the project
cargo build --release

# Create a symbolic link (Run as Administrator)
New-Item -ItemType SymbolicLink -Path "$env:USERPROFILE\.cargo\bin\qitops" -Target "$PWD\target\release\qitops-agent.exe"

# Verify installation
qitops --version
```

##### Linux/macOS

```bash
# Clone the repository
git clone https://github.com/jcopperman/qitops-agent.git
cd qitops-agent

# Build the project
cargo build --release

# Create a symbolic link
mkdir -p ~/.cargo/bin
ln -sf "$(pwd)/target/release/qitops-agent" ~/.cargo/bin/qitops

# Add to PATH if not already
echo 'export PATH="$HOME/.cargo/bin:$PATH"' >> ~/.bashrc
source ~/.bashrc

# Verify installation
qitops --version
```

#### Using Pre-built Binaries

Download the latest release from the [Releases page](https://github.com/jcopperman/qitops-agent/releases).

##### Windows

1. Download the Windows binary (`qitops-windows-amd64.exe`)
2. Rename it to `qitops.exe`
3. Move it to a directory in your PATH

##### Linux

```bash
# Download the latest release
curl -L https://github.com/jcopperman/qitops-agent/releases/latest/download/qitops-linux-amd64 -o qitops

# Make it executable
chmod +x qitops

# Move to a directory in your PATH
sudo mv qitops /usr/local/bin/
```

##### macOS

```bash
# Download the latest release
curl -L https://github.com/jcopperman/qitops-agent/releases/latest/download/qitops-macos-amd64 -o qitops

# Make it executable
chmod +x qitops

# Move to a directory in your PATH
sudo mv qitops /usr/local/bin/
```

#### Using Docker

```bash
# Pull the Docker image
docker pull jcopperman/qitops-agent:latest

# Run QitOps Agent in a container
docker run -it --rm jcopperman/qitops-agent:latest qitops --help
```

### Basic Commands

```bash
# Get help
qitops --help

# Generate test cases
qitops run test-gen --path src/user/auth.rs --format markdown

# Analyze a pull request
qitops run pr-analyze --pr 123

# Estimate risk of changes
qitops run risk --diff changes.diff

# Generate test data
qitops run test-data --schema user-profile --count 100

# Start an interactive testing session
qitops run session --name "Login Flow Test" --application "MyApp" --session-type exploratory --objectives "verify login,test error handling" --sources "documentation,code" --personas "tester,developer"
```

### Interactive Testing Sessions

QitOps Agent provides an interactive testing session feature that allows testers to have a conversation with an AI assistant to guide them through exploratory testing sessions:

```bash
# Start a basic session
qitops run session --name "Login Flow Test" --application "MyApp"

# Start a session with a specific type
qitops run session --name "Security Test" --application "MyApp" --session-type security

# Start a session with objectives, sources, and personas
qitops run session --name "User Journey Test" --application "MyApp" --session-type user-journey --objectives "verify checkout flow,test payment processing" --sources "documentation,code" --personas "new user,returning customer"
```

Session history is saved to the `sessions` directory for later reference.

### Monitoring and Metrics

QitOps Agent includes a built-in monitoring system that provides insights into its usage, performance, and resource consumption:

```bash
# Enable monitoring
export QITOPS_MONITORING_ENABLED=true
export QITOPS_MONITORING_HOST=127.0.0.1
export QITOPS_MONITORING_PORT=9090

# Start the monitoring stack
docker-compose -f docker-compose-monitoring.yml up -d

# Access the Grafana dashboard
open http://localhost:3000  # Default credentials: admin/qitops
```

See [Monitoring Documentation](docs/monitoring.md) for more details.

### LLM Management

QitOps Agent supports multiple LLM providers:

```bash
# List available providers
qitops llm list

# Add a new provider
qitops llm add --provider openai --api-key YOUR_API_KEY --model gpt-4

# Set default provider
qitops llm default --provider ollama

# Test a provider
qitops llm test --provider anthropic --prompt "Generate a test case for user authentication"
```

### GitHub Integration

QitOps Agent integrates with GitHub for PR analysis and risk assessment:

```bash
# Configure GitHub integration
qitops github config --token YOUR_GITHUB_TOKEN --owner username --repo repository

# Check GitHub configuration status
qitops github status

# Test GitHub connection
qitops github test

# Analyze a PR directly from GitHub
qitops run pr-analyze --pr https://github.com/username/repo/pull/123

# Assess risk from a GitHub PR
qitops run risk --diff https://github.com/username/repo/pull/123 --focus security,performance
```

### Configuration

QitOps Agent uses a configuration file located at:

- Windows: `%USERPROFILE%\.config\qitops\config.toml`
- Linux/macOS: `~/.config/qitops/config.toml`

You can also use environment variables to configure QitOps Agent:

```bash
# LLM configuration
export OPENAI_API_KEY="your-api-key"
export ANTHROPIC_API_KEY="your-api-key"
export OLLAMA_API_BASE="http://localhost:11434"

# GitHub configuration
export GITHUB_TOKEN="your-github-token"
export GITHUB_OWNER="your-username"
export GITHUB_REPO="your-repository"

# QitOps configuration
export QITOPS_DEFAULT_LLM_PROVIDER="ollama"
export QITOPS_DEFAULT_LLM_MODEL="mistral"
export QITOPS_SKIP_UPDATE_CHECK="true"
```

### Troubleshooting

If you encounter issues with QitOps Agent, try the following:

1. **Run with verbose logging**:
   ```bash
   qitops --verbose run test-gen --path src/user/auth.rs
   ```

2. **Check LLM configuration**:
   ```bash
   qitops llm list
   ```

3. **Test LLM connectivity**:
   ```bash
   qitops llm test --provider ollama
   ```

4. **Check GitHub configuration**:
   ```bash
   qitops github status
   ```

5. **Update to the latest version**:
   ```bash
   git pull
   cargo build --release
   ```

6. **Reset configuration**:
   ```bash
   # Windows
   Remove-Item -Recurse -Force "$env:USERPROFILE\.config\qitops"

   # Linux/macOS
   rm -rf ~/.config/qitops
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
