# OpenCode UX Analysis - Design Document

> **Author:** AI Analysis  
> **Date:** October 4, 2025  
> **Source:** https://github.com/sst/opencode  
> **Purpose:** Comprehensive analysis of OpenCode's agentic coding UX for `dong` improvements

---

## Table of Contents

- [Executive Summary](#executive-summary)
- [Architecture Overview](#architecture-overview)
- [Visual Design Patterns](#visual-design-patterns)
- [Message Rendering System](#message-rendering-system)
- [Input System UX](#input-system-ux)
- [Modal & Dialog System](#modal--dialog-system)
- [Session Management](#session-management)
- [Interaction Patterns](#interaction-patterns)
- [Theme System](#theme-system)
- [Performance Optimizations](#performance-optimizations)
- [Advanced Features](#advanced-features)
- [Implementation Roadmap for `dong`](#implementation-roadmap-for-dong)
- [Key Takeaways](#key-takeaways)

---

## Executive Summary

OpenCode is a terminal-based AI coding agent built with **Go + Bubbletea + Lipgloss**. This analysis reveals the UX patterns that make it feel polished and professional:

**Core Strengths:**

- ✅ **Component-based architecture** - Clean separation of concerns
- ✅ **Keyboard-first design** - Everything accessible via keyboard
- ✅ **Progressive disclosure** - Simple by default, power features opt-in
- ✅ **Real-time feedback** - Streaming responses, live updates
- ✅ **Smart caching** - Instant re-renders, no lag
- ✅ **Visual consistency** - Unified design language

**Technology Stack:**

- **Bubbletea v2** - Elm Architecture (Model/Update/View)
- **Lipgloss v2** - Styling system (similar to CSS-in-Go)
- **Bubbles** - Pre-built components (viewport, spinner, textarea)

---

## Architecture Overview

### Component Structure

```go
type Model struct {
    // Core Components
    editor    chat.EditorComponent       // Input area (bottom)
    messages  chat.MessagesComponent     // Chat history (scrollable)
    status    status.StatusComponent     // Status bar (fixed bottom)

    // Overlay Systems
    modal     layout.Modal               // Modal dialogs
    toastManager *toast.ToastManager     // Notifications
    completions  dialog.CompletionDialog // Autocomplete

    // State
    app       *app.App                   // Application state
    width, height int                    // Terminal dimensions
}
```

### Component Communication

Each component is **self-contained** with:

- `Init() tea.Cmd` - Initialization
- `Update(msg tea.Msg) (tea.Model, tea.Cmd)` - Message handling
- `View() string` - Rendering

**Message Passing:**

```go
// Components send messages via Cmd
return m, util.CmdHandler(app.SendPrompt{Text: "Hello"})

// Model routes messages to components
case app.SendPrompt:
    updated, cmd := m.messages.Update(msg)
    m.messages = updated
    return m, cmd
```

### Layout System

**Precise Positioning:**

```go
// Custom overlay system for exact X/Y positioning
mainLayout = layout.PlaceOverlay(
    x: 10,
    y: 5,
    foreground: content,
    background: mainLayout,
)
```

**Responsive Sizing:**

```go
container := min(width, 86)  // Max 86 chars wide
layout.Current = &layout.LayoutInfo{
    Viewport:  {Width: width, Height: height},
    Container: {Width: container},
}
```

---

## Visual Design Patterns

### 1. Home Screen (No Active Session)

```

█▀▀█ █▀▀█ █▀▀█ █▀▀▄     █▀▀▀ █▀▀█ █▀▀█ █▀▀█
█░░█ █░░█ █▀▀▀ █░░█     █░░░ █░░█ █░░█ █▀▀▀
▀▀▀▀ █▀▀▀ ▀▀▀▀ ▀  ▀     ▀▀▀▀ ▀▀▀▀ ▀▀▀▀ ▀▀▀▀
                                    v0.1.89

  /help     show all commands
  /new      start a new session
  /models   switch AI model
  /themes   change color theme

  ╭─────────────────────────────────────────╮
  │ > [cursor here]                         │
  ╰─────────────────────────────────────────╯

  enter send   gpt-4o
```

**Design Elements:**

- **Centered ASCII logo** (dual-tone: muted + emphasized)
- **Version display** (top right, only if width > 40)
- **Command palette** showing available `/` commands
- **Compact layout** (max 80 chars wide on home screen)
- **Input at bottom** with model indicator

### 2. Chat Screen (Active Session)

```
╭─────────────────────────────────────────────────────╮
│ # Debug authentication bug                          │
│  110K/23% ($0.15)            /share to create link  │
╰─────────────────────────────────────────────────────╯

[Scrollable message area]
│
│ User: Check the authentication logic @auth.rs
│
│ ╭─ gpt-4o ────────────────────────────────────────╮
│ │ I'll analyze the code. Let me check...          │
│ │                                                  │
│ │ ╭─ TOOL: read_file ──────────────────────────╮ │
│ │ │ file: src/auth.rs                          │ │
│ │ │ status: ✓ completed (0.8s)                 │ │
│ │ ╰────────────────────────────────────────────╯ │
│ │                                                  │
│ │ I found the issue. The token validation...      │
│ ╰──────────────────────────────────────────────────╯

╭─────────────────────────────────────────────────────╮
│ > Your prompt here_                                 │
╰─────────────────────────────────────────────────────╯
enter send        working... esc interrupt   gpt-4o

opencode v0.1.89  ~/project:main    MAIN AGENT
```

**Layout Structure:**

1. **Session Header** (bordered box)

   - Title as Markdown H1
   - Token counter: `110K/23% ($0.15)`
   - Share link hint

2. **Message Area** (scrollable viewport)

   - User messages
   - AI responses
   - Tool calls (nested boxes)
   - Thinking blocks (optional)

3. **Input Area** (bordered box)

   - Multi-line textarea
   - Prompt indicator: `>`
   - Dynamic border colors (accent/secondary/default)

4. **Status Bar** (fixed bottom)
   - Logo + version
   - Current directory + git branch
   - Active agent (color-coded)

---

## Message Rendering System

### User Messages

```
╭─ username ──────────────────────────────────────────╮
│ Check the token validation logic                    │
│                                                     │
│ txt auth.rs    img screenshot.png                   │
╰─────────────────────────────────────────────────────╯
```

**Features:**

- **File attachments** as colored badges
  - `txt` - Gray background (text files)
  - `img` - Accent color (images)
  - `pdf` - Primary color (PDFs)
- **Author name** from config
- **Queued indicator** if not yet processed

### Assistant Messages

```
╭─ gpt-4o ────────────────────────────────────────────╮
│ I found the issue. The token validation is missing  │
│ a null check before calling isEmpty().              │
│                                                     │
│ ╭─ TOOL: edit_file ──────────────────────────────╮  │
│ │ file: auth.rs                                  │  │
│ │ status: ✓ completed (2.3s)                     │  │
│ │                                                │  │
│ │ Changes:                                       │  │
│ │   + added null check                           │  │
│ │   + improved error message                     │  │
│ ╰────────────────────────────────────────────────╯  │
╰─────────────────────────────────────────────────────╯
```

**Elements:**

- **Model name** in header (e.g., "gpt-4o")
- **Markdown rendering** for formatted text
- **Tool calls** as nested boxes (if enabled)
- **Shimmer effect** on streaming text (90ms tick)
- **Completion status** with timing

### Thinking Blocks (Extended Reasoning)

```
╭─ 💭 gpt-4o thinking... ─────────────────────────────╮
│ Let me analyze the authentication flow:             │
│ 1. User submits credentials                         │
│ 2. Token generated                                  │
│ 3. Validation happens here <- BUG                   │
│                                                     │
│ The issue is that we're calling isEmpty() on a      │
│ potentially null token...                           │
╰─────────────────────────────────────────────────────╯
```

**Behavior:**

- Toggle with `/thinking` command
- Only **last streaming block** shimmers (not all)
- Collapsed by default
- Markdown formatted

### Tool Call Details

```
╭─ TOOL: edit_file ───────────────────────────────────╮
│ file: src/auth.rs                                   │
│ status: ✓ completed (2.3s)                          │
│                                                     │
│ ╭─ Diff ─────────────────────────────────────────╮  │
│ │ - if token.isEmpty():                          │  │
│ │ + if token == null or token.isEmpty():         │  │
│ ╰────────────────────────────────────────────────╯  │
╰─────────────────────────────────────────────────────╯
```

**States:**

- **In Progress:** Spinner + "running..."
- **Completed:** ✓ + execution time
- **Error:** ✗ + error message
- **Requires Permission:** Approval prompt

### Reverted Messages

```
╭─────────────────────────────────────────────────────╮
│ 2 messages reverted, 1 tool call reverted           │
│ ctrl+y (or /redo) to restore                        │
│                                                      │
│ Changes:                                            │
│   src/auth.rs  +12 -8                               │
│   tests/auth_test.rs  +5 -3                         │
╰──────────────────────────────────────────────────────╯
```

---

## Input System UX

### Input State Visualization

#### 1. Normal Mode

```
╭─────────────────────────────────────────────────────╮
│ > Your message here_                                │
╰─────────────────────────────────────────────────────╯
enter send    gpt-4o
```

- Border: Default color
- Prompt: `>`
- Shows current model

#### 2. Bash Mode (triggered by `!`)

```
╭─────────────────────────────────────────────────────╮
│ ! ls -la_                                           │
╰─────────────────────────────────────────────────────╯
enter run    esc cancel
```

- Border: Secondary color
- Prompt: `!`
- Hint: "enter run"

#### 3. Leader Sequence

```
╭─────────────────────────────────────────────────────╮
│ > _                                                 │
╰─────────────────────────────────────────────────────╯
[border glows accent color]
```

- Border: Accent color
- Waiting for next command key

#### 4. AI Working

```
╭─────────────────────────────────────────────────────╮
│ > _                                                 │
╰─────────────────────────────────────────────────────╯
working...    esc interrupt
```

- Spinner animation
- Shows interrupt hint

#### 5. Exit Confirmation (debounced)

```
╭─────────────────────────────────────────────────────╮
│ > _                                                 │
╰─────────────────────────────────────────────────────╯
esc again to exit
```

- Prevents accidental exit
- 1 second timeout

### Attachment System

**File Attachments:**

```
> Check this @auth.rs and @README.md_
```

Displays as:

```
> Check this [@auth.rs] and [@README.md]_
```

**Image Attachments:**

```
> Here's the screenshot [Image #1]_
```

- Paste images directly from clipboard
- Base64 encoded
- Numbered sequentially

**Long Text Auto-Summarize:**

```
> [pasted #1 50+ lines]_
```

- Triggers if > 3 lines OR > 150 chars
- Creates attachment instead of raw insert
- Configurable: `experimental.disablePasteSummary`

**Symbol References:**

```
> Fix the @validateToken function_
```

- LSP integration
- Autocomplete from workspace
- Precise context for AI

### History Navigation

```
Up Arrow / Ctrl+P    → Previous message in history
Down Arrow / Ctrl+N  → Next message in history
```

**Behavior:**

- Arrow keys only work at first/last line
- Ctrl+P/N work from anywhere
- Preserves current input when entering history
- Restores attachments from history
- Index: -1 = current, 0+ = history position

---

## Modal & Dialog System

### Completion Dialog

#### Command Completion (`/`)

```
╭─ Commands ──────────────────────────────────────────╮
│ > /he_                                              │
│                                                      │
│ ▸ /help          show all commands                  │
│   /history       view conversation history          │
╰──────────────────────────────────────────────────────╯
```

**Features:**

- Fuzzy search as you type
- Tab/Enter to select
- ESC to cancel
- Up/Down or Ctrl+P/N to navigate

#### File Completion (`@`)

```
╭─ Files ─────────────────────────────────────────────╮
│ > @auth_                                            │
│                                                      │
│ ▸ src/auth.rs         Authentication module         │
│   src/auth_test.rs    Auth tests                    │
│   config/auth.json    Auth configuration            │
╰──────────────────────────────────────────────────────╯
```

**Multiple Providers:**

- **Files** - Filesystem search
- **Symbols** - LSP workspace symbols
- **Agents** - Available agents

### Modal Dialogs

#### Help Dialog (`?` or `ctrl+?`)

```
╭─ Keybindings ───────────────────────────────────────╮
│ General                                             │
│   ctrl+?       show this help                       │
│   enter        send message                         │
│   ctrl+c       quit application                     │
│                                                     │
│ Navigation                                          │
│   ctrl+u       page up                              │
│   ctrl+d       page down                            │
│   g g          go to top                            │
│   G            go to bottom                         │
│                                                     │
│ Session                                             │
│   ctrl+n       new session                          │
│   ctrl+s       switch session                       │
│   ctrl+k       commands                             │
│                                                     │
│ [press esc to close]                                │
╰─────────────────────────────────────────────────────╯
```

#### Session List (`ctrl+s`)

```
╭─ Switch Session ────────────────────────────────────╮
│                                                     │
│ ▸ Debug authentication bug                          │
│   Implement user dashboard                          │
│   Fix database migration                            │
│   Add payment integration                           │
│                                                     │
│ [↑/↓/j/k navigate, enter select, esc close]         │
╰─────────────────────────────────────────────────────╯
```

#### Model Selection (`ctrl+o`)

```
╭─ Select Model ─────────────────────────────────────╮
│                                                    │
│ OpenAI                                             │
│ ▸ gpt-4o            128K context    $2.50/$10.00   │
│   gpt-4-turbo       128K context    $10.00/$30.00  │
│   gpt-3.5-turbo     16K context     $0.50/$1.50    │
│                                                    │
│ Anthropic                                          │
│   claude-3-opus     200K context    $15.00/$75.00  │
│   claude-3-sonnet   200K context    $3.00/$15.00   │
│                                                    │
│ [↑/↓ navigate, enter select, esc cancel]           │
╰────────────────────────────────────────────────────╯
```

**Shows:**

- Provider grouping
- Context window size
- Pricing (input/output per 1M tokens)
- Current model highlighted
- Recent models at top

---

## Session Management

### Session Header

```
╭─────────────────────────────────────────────────────╮
│ # Debug authentication bug                          │
│  110K/23% ($0.15)            /share to create link  │
╰─────────────────────────────────────────────────────╯
```

**Token Counter Format:**

- **Human-readable:** 110K (not 110,000)
- **Percentage:** 23% of context window (128K)
- **Cost:** Running total ($0.15)
- **Subscription models:** Hide cost, show only tokens/%

**Share Feature:**

```
Before: /share to create a shareable link
After:  https://opencode.ai/s/abc123    /unshare
```

### Session Compacting

OpenCode automatically manages context window usage:

**Auto-Compact Trigger:**

- Activates when tokens reach 95% of context window
- Requires `config.autoCompact` to be enabled
- Shows progress overlay during summarization

**Manual Compact:**

- Available via `/compact` command or command palette
- Summarizes current session
- Creates new session with summary as context

**Compacting UI:**

```
╭─────────────────────────────────────────────────────╮
│ Summarizing                                         │
│ Starting summarization...                           │
╰──────────────────────────────────────────────────────╯
```

> **Note:** Timeline view and undo/redo system were not found in the current OpenCode codebase and may be planned future features.

---

## Interaction Patterns

### Keyboard Navigation

**Message Scrolling:**

```
ctrl+u      page up (full page)
ctrl+d      page down (full page)
ctrl+b      half page up
ctrl+f      half page down
g g         go to top
G           go to bottom
```

**History:**

```
up/ctrl+p   previous message
down/ctrl+n next message
```

### Input Priority System

OpenCode routes input based on **strict priority**:

1. **Active modal** → Modal handles ALL input
2. **Permission prompt** → Only enter/a/esc work
3. **Completion dialog** → Dialog filters input
4. **Printable characters** → Editor gets immediate priority
5. **Leader sequence** → Wait for next command
6. **Global commands** → Check keybinds
7. **Fallback** → Send to editor

**Result:**

- Text input feels instant (no lag)
- Modals are inescapable (except ESC)
- Commands don't interfere with typing

---

## Theme System

### Auto-Detection

```go
case tea.BackgroundColorMsg:
    theme.UpdateSystemTheme(
        msg.Color,        // Terminal background
        msg.IsDark(),     // Light or dark?
    )
```

### Adaptive Colors

```go
type AdaptiveColor struct {
    Light string  // For light terminals
    Dark  string  // For dark terminals
}
```

**Example - Accent Color:**

- Light mode: Dark blue (#0066CC)
- Dark mode: Light blue (#66B3FF)

### Color Palette

Every theme defines:

```go
type Theme struct {
    Background()       Color  // Main background
    BackgroundPanel()  Color  // Panel background
    BackgroundElement() Color // Input/button background

    Text()       Color  // Primary text
    TextMuted()  Color  // Secondary text

    Primary()    Color  // Main brand color
    Secondary()  Color  // Alternative brand
    Accent()     Color  // Highlights

    Success()    Color  // Green
    Error()      Color  // Red
    Warning()    Color  // Yellow

    Border()     Color  // Border lines
}
```

### ANSI Fallback

For terminals without RGB support:

```go
if theme.CurrentThemeUsesAnsiColors() {
    output = util.ConvertRGBToAnsi16Colors(output)
}
```

---

## Performance Optimizations

### 1. Render Caching

```go
type PartCache struct {
    cache map[string]string  // key → rendered output
}
```

**Cache Keys Include:**

- Message ID
- Content hash
- Width
- Settings (tool details, etc.)

**Benefits:**

- Instant re-renders when scrolling
- Width changes clear cache (for reflow)
- Only re-render changed messages

**Implementation:**

```go
key := cache.GenerateKey(messageID, text, width, settings)
if content, cached := cache.Get(key); cached {
    return content  // Cache hit!
}
content = renderMessage(...)
cache.Set(key, content)
```

### 2. Async Rendering

```go
func (m *messagesComponent) renderView() tea.Cmd {
    if m.rendering {
        m.dirty = true  // Mark for re-render after
        return nil      // Skip this render
    }
    m.rendering = true

    return func() tea.Msg {
        // Heavy rendering work in goroutine
        content := renderAllMessages()
        return renderCompleteMsg{content: content}
    }
}
```

**Prevents:**

- UI freezing during long renders
- Dropped frames
- Input lag

### 3. Shimmer Animation Optimization

Only the **last streaming reasoning block** shimmers:

```go
lastStreamingReasoningID := ""
for mi := len(messages) - 1; mi >= 0; mi-- {
    // Find last incomplete reasoning part
    if reasoningPart.Time.End == 0 {
        lastStreamingReasoningID = reasoningPart.ID
        break
    }
}
```

**90ms tick** for smooth animation without CPU overhead.

### 4. Event Streaming

```go
go func() {
    stream := httpClient.Event.ListStreaming(ctx, params)
    for stream.Next() {
        program.Send(stream.Current())
    }
}()
```

**Real-time updates** without polling:

- Message parts arrive incrementally
- Tool status updates stream in
- Session changes sync instantly
- No polling overhead

---

## Advanced Features

### Permission Prompts

```
╭────────────────────────────────────────────────────╮
│ The AI wants to run:                               │
│                                                    │
│ ╭─ TOOL: shell ──────────────────────────────────╮ │
│ │ command: rm -rf node_modules                   │ │
│ │                                                │ │
│ │ [enter] approve once                           │ │
│ │ [a]     approve always                         │ │
│ │ [esc]   reject                                 │ │
│ ╰────────────────────────────────────────────────╯ │
╰────────────────────────────────────────────────────╯
```

**Behavior:**

- Blocks AI execution until approved
- Editor loses focus (can't type)
- Clear keyboard shortcuts
- "approve always" remembers for session
- Queues multiple permissions

### Toast Notifications

**Success:**

```
┌────────────────────────────────────┐
│ ✓ Session deleted successfully     │
└────────────────────────────────────┘
```

**Error:**

```
┌────────────────────────────────────┐
│ ✗ Failed to connect to server      │
└────────────────────────────────────┘
```

**Info:**

```
┌────────────────────────────────────┐
│ ℹ Tool details are now visible     │
└────────────────────────────────────┘
```

**Features:**

- Auto-dismiss after 3 seconds
- Stack multiple toasts
- Non-blocking (overlay)
- Color-coded by type
- Top-right corner placement

### Child Sessions (Agent Tasks)

OpenCode supports nested sessions via the Agent tool:

**Implementation:**

```go
// Agent tool creates child sessions
if toolCall.Name == agent.AgentToolName {
    taskMessages, _ := messagesService.List(context.Background(), toolCall.ID)
    // Renders nested tool calls from child session
}
```

**Visual Representation:**

- Tool calls from child sessions are rendered nested within the parent
- Shows hierarchical structure of task execution
- Each subtask rendered with indentation

**Use Cases:**

- Complex multi-step operations
- Parallel task execution
- Isolated context for specific operations

---

## Implementation Roadmap for `dong`

### Phase 1: Foundation (Week 1-2)

**Priority: Component Architecture**

```rust
// Current structure
src/chat.rs (789 lines - monolithic)

// Target structure
src/chat/
  mod.rs           // Coordinator
  messages.rs      // Message rendering
  editor.rs        // Input handling
  status.rs        // Status bar
  modal.rs         // Modal system
  types.rs         // Shared types
```

**Benefits:**

- Easier to maintain
- Clearer responsibilities
- Parallel development
- Better testing

**Implementation:**

1. Create `chat/` directory
2. Extract `MessageType`, `Sender`, `Message` → `types.rs`
3. Extract message rendering → `messages.rs`
4. Extract input handling → `editor.rs`
5. Create status bar → `status.rs`
6. Update `mod.rs` to coordinate

---

### Phase 2: Core UX (Week 3-4)

#### A. Toast Notification System

```rust
pub struct ToastManager {
    toasts: VecDeque<Toast>,
    auto_dismiss: HashMap<usize, Instant>,
}

pub enum ToastType {
    Success,
    Error,
    Info,
    Warning,
}

impl ToastManager {
    pub fn show(&mut self, message: String, toast_type: ToastType) {
        // Add to queue
        // Start auto-dismiss timer (3s)
    }

    pub fn render(&self, base: String) -> String {
        // Overlay toasts in top-right corner
    }
}
```

**Quick Win:** Big UX improvement, low implementation cost

#### B. Status Bar Component

```rust
pub struct StatusBar {
    version: String,
    cwd: String,
    git_branch: Option<String>,
    model_name: String,
}

impl StatusBar {
    pub fn render(&self, width: u16) -> String {
        // Left: logo + version + cwd:branch
        // Right: model name
        // Bottom of screen, always visible
    }
}
```

**Layout:**

```
opencode v0.1.89  ~/project:main    gpt-4o
```

#### C. Theme Detection

```rust
pub struct Theme {
    background: Color,
    text: Color,
    text_muted: Color,
    primary: Color,
    // ...
    is_dark: bool,
}

impl Theme {
    pub fn detect() -> Self {
        // Check $COLORFGBG env var
        // Or use ratatui backend.get_color()
        // Default to dark if unknown
    }

    pub fn from_terminal_background(bg: Color) -> Self {
        let is_dark = is_color_dark(bg);
        if is_dark {
            Self::dark()
        } else {
            Self::light()
        }
    }
}
```

---

### Phase 3: Input Enhancements (Week 5-6)

#### A. Attachment System

```rust
pub enum Attachment {
    File { path: PathBuf, display: String },
    Image { data: Vec<u8>, display: String },
    Text { content: String, display: String },
    Symbol { name: String, location: Location },
}

pub struct ReactiveTextbox {
    content: String,
    attachments: Vec<Attachment>,
    cursor: usize,
}

impl ReactiveTextbox {
    pub fn insert_attachment(&mut self, att: Attachment) {
        // Insert at cursor position
        // Replace text with display token
    }

    pub fn get_attachments(&self) -> &[Attachment] {
        &self.attachments
    }
}
```

#### B. History Navigation

```rust
pub struct InputHistory {
    history: Vec<String>,
    index: isize,  // -1 = current, 0+ = history
    current_text: String,
}

impl InputHistory {
    pub fn previous(&mut self, current: String) -> Option<String> {
        if self.index == -1 {
            self.current_text = current;
        }
        self.index = min(self.index + 1, self.history.len() - 1);
        self.history.get(self.index).cloned()
    }

    pub fn next(&mut self) -> Option<String> {
        if self.index == -1 {
            return None;
        }
        self.index -= 1;
        if self.index == -1 {
            Some(self.current_text.clone())
        } else {
            self.history.get(self.index).cloned()
        }
    }
}
```

#### C. Completion Dialog

```rust
pub struct CompletionDialog {
    items: Vec<CompletionItem>,
    filtered: Vec<usize>,
    selected: usize,
    query: String,
}

pub struct CompletionItem {
    display: String,
    value: String,
    description: String,
    category: String,
}

impl CompletionDialog {
    pub fn filter(&mut self, query: &str) {
        // Fuzzy match against items
        // Update filtered indices
    }

    pub fn render(&self) -> String {
        // Show filtered items
        // Highlight selected
        // Group by category
    }
}
```

---

### Phase 4: Session Features (Week 7-8)

#### A. Session List Modal

```rust
pub struct SessionListModal {
    sessions: Vec<Session>,
    selected: usize,
    filter: String,
}

pub struct Session {
    id: String,
    title: String,
    timestamp: SystemTime,
    message_count: usize,
}

impl SessionListModal {
    pub fn render(&self) -> String {
        // List sessions with relative time
        // Show: title, timestamp, message count
        // Actions: select, new, rename, delete
    }
}
```

#### B. Timeline View

```rust
pub struct TimelineView {
    messages: Vec<Message>,
    selected: usize,
}

impl TimelineView {
    pub fn render(&self) -> String {
        // Tree structure:
        // User message
        //   ├─ AI response
        //   ├─ Tool call
        //   └─ AI followup
        // User message
        //   └─ AI response
    }

    pub fn scroll_to(&self, message_id: &str) {
        // Jump to message in main view
    }

    pub fn restore_to(&self, message_id: &str) {
        // Revert conversation to this point
    }
}
```

#### C. Undo/Redo

```rust
pub struct UndoRedo {
    history: Vec<ConversationState>,
    position: usize,
}

pub struct ConversationState {
    messages: Vec<Message>,
    files_changed: HashMap<PathBuf, String>,
}

impl UndoRedo {
    pub fn undo(&mut self) -> Option<ConversationState> {
        if self.position > 0 {
            self.position -= 1;
            Some(self.history[self.position].clone())
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<ConversationState> {
        if self.position < self.history.len() - 1 {
            self.position += 1;
            Some(self.history[self.position].clone())
        } else {
            None
        }
    }
}
```

---

### Phase 5: Polish (Week 9-10)

#### A. Permission Prompts

```rust
pub struct PermissionPrompt {
    tool_name: String,
    command: String,
    args: Vec<String>,
}

impl PermissionPrompt {
    pub fn render(&self) -> String {
        // Show tool details
        // Display: [enter] once, [a] always, [esc] reject
    }

    pub fn handle_input(&mut self, key: KeyCode) -> PermissionResult {
        match key {
            KeyCode::Enter => PermissionResult::ApproveOnce,
            KeyCode::Char('a') => PermissionResult::ApproveAlways,
            KeyCode::Esc => PermissionResult::Reject,
            _ => PermissionResult::None,
        }
    }
}
```

#### B. Tool Details (Collapsible)

```rust
pub struct ToolCall {
    id: String,
    tool: String,
    args: serde_json::Value,
    status: ToolStatus,
    result: Option<String>,
    duration: Option<Duration>,
}

pub enum ToolStatus {
    InProgress,
    Completed,
    Error(String),
}

impl ToolCall {
    pub fn render(&self, collapsed: bool) -> String {
        if collapsed {
            // Single line: tool name + status
        } else {
            // Full box with args, result, timing
        }
    }
}
```

#### C. Thinking Blocks

```rust
pub struct ThinkingBlock {
    id: String,
    text: String,
    streaming: bool,
}

impl ThinkingBlock {
    pub fn render(&self, shimmer: bool) -> String {
        if shimmer && self.streaming {
            // Apply shimmer effect (color cycling)
        } else {
            // Normal markdown rendering
        }
    }
}
```

---

## Key Takeaways

### What Makes OpenCode's UX Excellent

1. **Keyboard-First Design**

   - Everything accessible via keyboard
   - Mouse is optional enhancement
   - Vi/Emacs hybrid keybinds

2. **Smart Input Routing**

   - Text input never feels laggy
   - Modals properly capture focus
   - Commands don't interfere with typing

3. **Progressive Disclosure**

   - Simple by default
   - Power features opt-in
   - Clean interface for beginners

4. **Visual Consistency**

   - Same border styles everywhere
   - Predictable color usage
   - Unified component look

5. **Performance Focus**

   - Aggressive caching
   - Async rendering
   - No janky animations

6. **Real-time Feedback**

   - Streaming responses
   - Live status updates
   - Immediate confirmations

7. **Error Resilience**
   - Never crash UI
   - Always show feedback
   - Graceful degradation

### Critical Success Factors

**Architecture:**

- Component-based design
- Clear separation of concerns
- Message-passing communication

**UX Patterns:**

- Immediate visual feedback
- Non-blocking operations
- Context-aware hints

**Performance:**

- Render caching
- Async operations
- Optimized animations

**Accessibility:**

- Keyboard-first
- High contrast themes
- Screen reader friendly

---

## Recommended Next Steps for `dong`

### Immediate (Week 1)

1. **Split chat.rs** into component modules
2. **Add toast notifications** (quick win)
3. **Implement status bar**

### Short-term (Weeks 2-4)

4. **Add theme detection**
5. **Implement attachment system** (max 5, with delete mode)
6. **Add history navigation**
7. **Create completion dialog**

### Medium-term (Weeks 5-8)

8. **Session list modal** (with vim navigation)
9. **Model selection dialog**
10. **Command palette** (custom commands)
11. **External editor integration** (`ctrl+e`)

### Long-term (Weeks 9-12)

12. **Permission prompts** (tool-specific rendering)
13. **Tool details UI** (nested rendering)
14. **Thinking blocks** (with streaming)
15. **Performance optimizations** (caching, async)
16. **Multi-arguments dialog** (parameterized commands)

---

## Conclusion

OpenCode demonstrates that a **terminal UI can feel as polished as a GUI** with:

- **Thoughtful component architecture**
- **Attention to visual detail**
- **Smart performance optimizations**
- **Progressive feature disclosure**
- **Consistent interaction patterns**

By applying these patterns to `dong`, we can create a **best-in-class agentic coding experience** in Rust + Ratatui.

---

**References:**

- OpenCode: https://github.com/opencode-ai/opencode
- Bubbletea: https://github.com/charmbracelet/bubbletea
- Lipgloss: https://github.com/charmbracelet/lipgloss
- Ratatui: https://github.com/ratatui-org/ratatui
