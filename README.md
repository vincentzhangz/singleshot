# singleshot

A CLI tool for testing AI models with a single prompt. Supports multiple providers including OpenAI, Anthropic, Ollama, and OpenRouter.

## Features

- **Multiple Providers**: OpenAI, Anthropic, Ollama, OpenRouter
- **Vision Support**: Send images with your prompts
- **Config Files**: Save and reuse prompt configurations
- **Reports**: Generate markdown reports of your tests
- **Flexible Input**: Prompt via argument, file, config, or stdin

## Installation

```bash
cargo install singleshot
```

Or build from source:

```bash
git clone https://github.com/vincentzhangz/singleshot
cd singleshot
cargo install --path .
```

## Quick Start

```bash
# Simple prompt with Ollama (default)
singleshot chat -p "What is Rust?"

# Use OpenAI
singleshot chat -p "Explain async/await" -P openai

# Use a specific model
singleshot chat -p "Hello" -P anthropic -m claude-sonnet-4-20250514

# Send an image
singleshot chat -p "What's in this image?" -P openai -i photo.jpg
```

## Usage

### Commands

| Command    | Description                          |
| ---------- | ------------------------------------ |
| `chat`     | Send a prompt to an AI model         |
| `models`   | List available models for a provider |
| `ping`     | Test connection to a provider        |
| `generate` | Generate an example config file      |

### Chat Options

```
-p, --prompt <PROMPT>        The prompt to send
-f, --file <FILE>            File containing the prompt
-l, --load <LOAD>            Load config from markdown file
-i, --image <IMAGE>          Image file to include
-v, --video <VIDEO>          Video file to include
-a, --audio <AUDIO>          Audio file to include
-P, --provider <PROVIDER>    AI provider [default: ollama]
-m, --model <MODEL>          Model name
-b, --base-url <BASE_URL>    Custom API base URL
-s, --system <SYSTEM>        System prompt
-t, --temperature <TEMP>     Temperature (0.0-2.0) [default: 0.7]
    --max-tokens <TOKENS>    Maximum tokens in response
-d, --detail                 Show detailed information
-r, --report [<FILE>]        Generate report file
```

### Providers

| Provider     | Default Model            | Environment Variable |
| ------------ | ------------------------ | -------------------- |
| `ollama`     | llama3.2                 | -                    |
| `openai`     | gpt-4o                   | `OPENAI_API_KEY`     |
| `anthropic`  | claude-sonnet-4-20250514 | `ANTHROPIC_API_KEY`  |
| `openrouter` | openai/gpt-4o            | `OPENROUTER_API_KEY` |

## Examples

### Read prompt from file

```bash
singleshot chat -f prompt.txt -P openai
```

### Pipe from stdin

```bash
echo "Summarize this" | singleshot chat -P openai
```

### Use a config file

Create a config file with `singleshot generate -o config.md`:

```markdown
---provider---
openai

---model---
gpt-4o

---base_url---
https://api.openai.com/v1

---temperature---
0.7

---max_tokens---
1024

---system---
You are a helpful coding assistant.

---prompt---
Explain the difference between async and threads.

---file---
path/to/context.txt

---image---
path/to/image.jpg

---video---
path/to/video.mp4

---audio---
path/to/audio.mp3
```

All sections are optional. Only include what you need.

Then run:

```bash
singleshot chat -l config.md
```

### Generate a report

```bash
singleshot chat -p "What is Rust?" -P openai -r report.md
```

### Vision with detailed output

```bash
singleshot chat -p "Describe this image" -i photo.jpg -P openai -d -r
```

### List available models

```bash
singleshot models -P openai
singleshot models -P ollama
```

### Test provider connection

```bash
singleshot ping -P openai
singleshot ping -P ollama
```

## Environment Variables

Set API keys for cloud providers:

```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
export OPENROUTER_API_KEY="sk-or-..."
```

For custom endpoints:

```bash
export OPENAI_BASE_URL="https://your-proxy.com/v1"
export OLLAMA_API_BASE_URL="http://remote-server:11434"
```

## License

Code released under the [MIT License](https://github.com/vincentzhangz/singleshot/blob/main/LICENSE).
