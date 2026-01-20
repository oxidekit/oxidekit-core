# OxideKit: Honest Gap Assessment vs Tauri/Electron/Flutter

**Last Updated:** 2026-01-19
**Version:** 0.3.0

This document provides a brutally honest assessment of what OxideKit is missing compared to production-ready frameworks like Tauri, Electron, and Flutter. Every item here represents real work needed before developers can ship production applications.

---

## 1. Critical Gaps (Blocking Production Use)

These are **must-have** features that prevent shipping any real application.

### 1.1 Event Handling (COMPLETELY MISSING)

**Current State:** The runtime processes `WindowEvent` from winit but does NOT propagate events to UI components.

| Gap | Tauri/Electron | Flutter | OxideKit |
|-----|----------------|---------|----------|
| onClick/onPress | Full support | Full support | **MISSING** - specs exist in `oxide-components/src/spec.rs` but no runtime implementation |
| onHover/onMouseEnter | Full support | Full support | **MISSING** |
| onFocus/onBlur | Full support | Full support | **MISSING** |
| onKeyDown/onKeyUp | Full support | Full support | **MISSING** |
| onScroll | Full support | Full support | **MISSING** - layout has scroll containers but no event handling |
| onDrag/onDrop | Full support | Full support | **MISSING** |
| Touch events | Full support | Full support | **MISSING** |
| Gesture recognition | N/A | Full support | **MISSING** |

**Impact:** Cannot build any interactive UI. Buttons don't click, forms don't submit.

**Files to modify:**
- `oxide-runtime/src/lib.rs` - needs event propagation in `window_event()`
- New: `oxide-runtime/src/events.rs` - event dispatch system
- New: `oxide-runtime/src/hit_test.rs` - hit testing for mouse events

### 1.2 Text Input & Editing (COMPLETELY MISSING)

**Current State:** `oxide-text` handles rendering only. No input, cursor, or selection.

| Gap | Tauri/Electron | Flutter | OxideKit |
|-----|----------------|---------|----------|
| Single-line text input | Full | Full | **MISSING** |
| Multi-line text area | Full | Full | **MISSING** |
| Cursor positioning | Full | Full | **MISSING** |
| Text selection | Full | Full | **MISSING** |
| Copy/Cut/Paste | Full | Full | **MISSING** |
| IME support (CJK, etc.) | Full | Full | Specs in `oxide-mobile/src/ime/` but **NO IMPLEMENTATION** |
| Undo/Redo | Full | Full | **MISSING** |
| Password masking | Full | Full | **MISSING** |
| Auto-complete | Full | Full | **MISSING** |

**Impact:** Cannot build any form, search bar, or text-based application.

**Files to create:**
- `oxide-text/src/input.rs` - text input state machine
- `oxide-text/src/selection.rs` - selection handling
- `oxide-text/src/ime.rs` - IME integration
- `oxide-runtime/src/focus.rs` - focus management

### 1.3 IPC / Process Communication (COMPLETELY MISSING)

**Current State:** No mechanism for Rust backend <-> UI communication.

| Gap | Tauri | Electron | Flutter | OxideKit |
|-----|-------|----------|---------|----------|
| Command invocation | `#[tauri::command]` | `ipcMain/ipcRenderer` | Platform channels | **MISSING** |
| Async commands | Full | Full | Full | **MISSING** |
| Events from backend | Full | Full | Full | **MISSING** |
| Streaming data | Full | Full | Full | **MISSING** |
| Type-safe bindings | Full | Manual | Auto-gen | **MISSING** |

**Impact:** UI is completely isolated from application logic. Cannot load data, save state, or perform any business logic.

**Files to create:**
- `oxide-ipc/` - new crate for IPC
- `oxide-ipc/src/command.rs` - command system
- `oxide-ipc/src/invoke.rs` - invoke mechanism
- `oxide-ipc-macros/` - `#[oxide::command]` proc macro

### 1.4 Window Management APIs (PARTIAL)

**Current State:** Basic window creation via winit. No multi-window, no control APIs.

| Gap | Tauri | Electron | Flutter | OxideKit |
|-----|-------|----------|---------|----------|
| Create window | Full | Full | N/A | Basic only |
| Multi-window | Full | Full | N/A | **MISSING** |
| Window position/size | Full | Full | N/A | Read-only |
| Minimize/Maximize | Full | Full | N/A | **MISSING** |
| Fullscreen | Full | Full | N/A | **MISSING** |
| Always on top | Full | Full | N/A | **MISSING** |
| Window close prevention | Full | Full | N/A | **MISSING** |
| Frameless/transparent | Full | Full | N/A | **MISSING** |
| Title bar customization | Full | Full | N/A | **MISSING** |
| Window state persistence | Full | Manual | N/A | **MISSING** |

**Files to modify:**
- `oxide-runtime/src/lib.rs` - extend `WindowConfig`
- New: `oxide-runtime/src/window_manager.rs`

### 1.5 File System Access (MISSING FROM RUNTIME)

**Current State:** `oxide-permissions` has filesystem capability specs. No actual API.

| Gap | Tauri | Electron | Flutter | OxideKit |
|-----|-------|----------|---------|----------|
| Read file | Full | Full | Plugin | Specs only, **NO API** |
| Write file | Full | Full | Plugin | Specs only, **NO API** |
| File dialogs (open/save) | Full | Full | Plugin | Specs only, **NO API** |
| Directory listing | Full | Full | Plugin | Specs only, **NO API** |
| File watching | Full | Full | Plugin | Specs only, **NO API** |
| Path APIs | Full | Full | Plugin | **MISSING** |
| Sandboxed access | Full | Manual | Manual | Designed but **NOT IMPLEMENTED** |

**Files to create:**
- `oxide-fs/` - new crate
- `oxide-fs/src/api.rs` - file system API
- `oxide-fs/src/dialog.rs` - native dialogs

### 1.6 System Tray (MISSING)

**Current State:** Not implemented. Specs mention it in capability docs only.

| Gap | Tauri | Electron | Flutter | OxideKit |
|-----|-------|----------|---------|----------|
| Tray icon | Full | Full | Plugin | **MISSING** |
| Tray menu | Full | Full | Plugin | **MISSING** |
| Tray events | Full | Full | Plugin | **MISSING** |
| Badge/overlay | Full | Full | Plugin | **MISSING** |

### 1.7 Native Notifications (MISSING)

**Current State:** Capability specs exist. No implementation.

| Gap | Tauri | Electron | Flutter | OxideKit |
|-----|-------|----------|---------|----------|
| Show notification | Full | Full | Plugin | **MISSING** |
| Actions/buttons | Full | Full | Plugin | **MISSING** |
| Click handling | Full | Full | Plugin | **MISSING** |
| Scheduling | Manual | Manual | Plugin | **MISSING** |

### 1.8 Native Menus (MISSING)

**Current State:** Not implemented.

| Gap | Tauri | Electron | Flutter | OxideKit |
|-----|-------|----------|---------|----------|
| Application menu | Full | Full | N/A | **MISSING** |
| Context menu | Full | Full | N/A | **MISSING** |
| Keyboard shortcuts | Full | Full | Manual | **MISSING** |

### 1.9 Clipboard (MISSING)

**Current State:** Mentioned in portable API categories. Not implemented.

| Gap | Tauri | Electron | Flutter | OxideKit |
|-----|-------|----------|---------|----------|
| Read text | Full | Full | Plugin | **MISSING** |
| Write text | Full | Full | Plugin | **MISSING** |
| Read image | Full | Full | Plugin | **MISSING** |
| Write image | Full | Full | Plugin | **MISSING** |
| Rich text/HTML | Full | Full | Limited | **MISSING** |

---

## 2. Important Gaps (Degraded Experience)

These gaps won't block shipping but will result in a noticeably worse user experience.

### 2.1 Animations (PARTIAL)

**Current State:** `oxide-components/src/animation/` has a full animation system. `oxide-runtime/src/animation.rs` integrates it. **But it's not connected to rendering.**

| Gap | Issue |
|-----|-------|
| Animation -> Render pipeline | Animation values computed but never applied to visuals |
| CSS-like transitions | Defined but not triggered on property changes |
| Micro-interactions | No hover/focus state animations |
| Page transitions | Not implemented |
| Skeleton loading | Component exists but no animation |
| Gesture-driven animations | No gesture system |

**Files to fix:**
- `oxide-runtime/src/lib.rs` - connect `AnimationRuntime` to render loop
- `oxide-runtime/src/animation.rs` - apply values to `NodeVisual`

### 2.2 Accessibility (SPECS ONLY)

**Current State:** `oxide-quality/src/a11y.rs` has comprehensive WCAG validation. No runtime accessibility.

| Gap | Issue |
|-----|-------|
| Screen reader support | **MISSING** - no accessibility tree |
| Keyboard navigation | **MISSING** - no focus management |
| Focus indicators | Specs only, no rendering |
| ARIA roles at runtime | **MISSING** |
| High contrast mode | **MISSING** |
| Reduced motion | Design tokens exist, not enforced |
| VoiceOver/TalkBack | **MISSING** |

**This is a legal requirement for many applications (ADA, WCAG compliance).**

### 2.3 Hot Reload for Native (PARTIAL)

**Current State:** `oxide-devtools/src/hot_reload/` has file watching and state management. UI reload is **not connected**.

| Gap | Issue |
|-----|-------|
| .oui file reload | Detected but not applied to running app |
| Rust code reload | Not possible (recompile required) |
| State preservation | `StateManager` exists but not integrated |
| Error overlay | Exists but not rendered |

### 2.4 Theming & Dark Mode (PARTIAL)

**Current State:** Design tokens and theme system exist. No OS integration.

| Gap | Issue |
|-----|-------|
| OS dark mode detection | **MISSING** |
| Automatic theme switching | **MISSING** |
| System accent colors | **MISSING** |
| Theme hot-swap | Not implemented |

### 2.5 Responsive Layout (PARTIAL)

**Current State:** `oxide-layout/src/responsive.rs` has breakpoints. Not integrated with runtime.

| Gap | Issue |
|-----|-------|
| Breakpoint detection | Exists but not connected to window resize |
| Responsive values | API exists, not applied |
| Safe area insets | Specs exist, not implemented |

### 2.6 Image Loading (MISSING)

**Current State:** No image component or loading system.

| Gap | Issue |
|-----|-------|
| Load from file | **MISSING** |
| Load from URL | **MISSING** |
| Image decoding | **MISSING** |
| Lazy loading | **MISSING** |
| Caching | **MISSING** |
| SVG support | **MISSING** |

### 2.7 Lists & Virtualization (MISSING)

**Current State:** No virtualized list component.

| Gap | Issue |
|-----|-------|
| Virtual scrolling | **MISSING** |
| Recycled views | **MISSING** |
| Infinite scroll | **MISSING** |
| Pull to refresh | **MISSING** |

---

## 3. Nice to Have (Polish & DX)

These are quality-of-life improvements that make the difference between good and great.

### 3.1 Plugin Ecosystem (INFRASTRUCTURE ONLY)

**Current State:** `oxide-plugins` has manifest, sandbox, permissions, registry. No actual plugins.

| Gap | Issue |
|-----|-------|
| Plugin loading | Loader exists but no runtime hook |
| WASM sandbox | Wasmtime integration planned, not done |
| First-party plugins | None published |
| Plugin marketplace | Registry schema only |

### 3.2 IDE Integrations (PARTIAL)

**Current State:** `oxide-lsp` exists with basic completion/hover. VS Code extension planned.

| Gap | Issue |
|-----|-------|
| .oui syntax highlighting | In VS Code extension repo, not published |
| Go to definition | Implemented in LSP |
| Auto-completion | Basic implementation |
| Error squiggles | Implemented |
| Code actions | **MISSING** |
| Refactoring | **MISSING** |
| Debugger integration | **MISSING** |

### 3.3 Developer Tools (PARTIAL)

**Current State:** Inspector exists in `oxide-devtools`. Not launchable.

| Gap | Issue |
|-----|-------|
| Element inspector | Code exists, no UI |
| Layout debugger | Debug overlay works |
| Performance profiler | Metrics exist, no UI |
| Network inspector | **MISSING** |
| State inspector | **MISSING** |

### 3.4 Testing Infrastructure (MISSING)

| Gap | Issue |
|-----|-------|
| Component testing | **MISSING** |
| Snapshot testing | **MISSING** |
| Visual regression | Mentioned in Task 34, **NOT IMPLEMENTED** |
| E2E testing | **MISSING** |
| Accessibility testing | a11y checks are static, not runtime |

### 3.5 Production Build Optimizations (PARTIAL)

| Gap | Issue |
|-----|-------|
| Tree shaking | **MISSING** |
| Asset optimization | **MISSING** |
| Bundle splitting | **MISSING** |
| Compression | **MISSING** |

---

## 4. What OxideKit Has That Others Don't

These are genuine differentiators worth preserving and highlighting.

### 4.1 Compile-Time Validation

- **Component spec validation** - Props, events, slots validated at build time
- **Design token enforcement** - Type-safe token references
- **.oui type checking** - Compiler catches errors before runtime

**Tauri/Electron:** Runtime errors only
**Flutter:** Some compile-time but mostly runtime

### 4.2 Design Token System

- **First-class tokens** - Colors, spacing, typography, motion, shadows, radius
- **Theme export/import** - TOML format, Figma sync
- **AI-exportable schema** - `oxide.ai.json` for AI tools

**Tauri/Electron:** CSS variables, no structure
**Flutter:** ThemeData, less comprehensive

### 4.3 Portability Checking

- **API-level portability** - `oxide-portable` marks APIs by platform
- **Build-time warnings** - Catches iOS-only code in desktop builds
- **Alternative suggestions** - Recommends portable APIs

**Tauri/Electron:** Runtime crashes
**Flutter:** Compile errors only

### 4.4 Security Model

- **Capability-based permissions** - Declared in manifest, enforced
- **Sandboxed plugins** - WASM isolation (when implemented)
- **Attestation support** - Code signing verification

**Tauri:** Similar capability model
**Electron:** All or nothing

### 4.5 No JavaScript Runtime

- **Pure Rust** - No V8, no bundler, no node_modules
- **Smaller binaries** - No 100MB Chromium
- **Consistent performance** - No GC pauses

**Electron:** 100MB+ binaries
**Tauri:** Small but still needs JS

### 4.6 Native Rendering

- **GPU-accelerated** - wgpu for all rendering
- **No DOM** - Direct to GPU, no layout thrashing
- **Consistent across platforms** - Same pixel output

**Electron:** Chromium's renderer
**Flutter:** Similar (Skia)

---

## 5. Roadmap to Feature Parity

A realistic week-by-week plan to reach production readiness.

### Phase 1: Core Interactivity (Weeks 1-4)

**Week 1: Event System Foundation**
- [ ] Implement hit testing in `oxide-runtime`
- [ ] Add event dispatch loop
- [ ] Connect mouse events (click, hover, enter, leave)
- [ ] Basic onClick handler propagation

**Week 2: Focus & Keyboard**
- [ ] Implement focus management
- [ ] Tab navigation
- [ ] Keyboard event dispatch
- [ ] onFocus/onBlur handlers

**Week 3: Text Input Basics**
- [ ] Single-line TextInput component
- [ ] Cursor rendering
- [ ] Basic typing
- [ ] Backspace/delete

**Week 4: Text Selection & Clipboard**
- [ ] Text selection
- [ ] Shift+arrow selection
- [ ] Copy/paste via system clipboard
- [ ] Cut support

### Phase 2: Application Shell (Weeks 5-8)

**Week 5: IPC Foundation**
- [ ] Create `oxide-ipc` crate
- [ ] Command definition macro
- [ ] Invoke mechanism
- [ ] Basic command execution

**Week 6: IPC Async & Events**
- [ ] Async command support
- [ ] Backend-to-UI events
- [ ] Type-safe bindings generation
- [ ] Error handling

**Week 7: Window Management**
- [ ] Multi-window support
- [ ] Window position/size APIs
- [ ] Minimize/maximize/fullscreen
- [ ] Window events

**Week 8: File System**
- [ ] File read/write APIs
- [ ] Native open/save dialogs
- [ ] Directory operations
- [ ] Path utilities

### Phase 3: Platform Features (Weeks 9-12)

**Week 9: System Tray & Notifications**
- [ ] System tray icon
- [ ] Tray menu
- [ ] Native notifications
- [ ] Click handling

**Week 10: Menus & Clipboard**
- [ ] Application menu (macOS menu bar)
- [ ] Context menus
- [ ] Keyboard shortcuts
- [ ] Full clipboard API

**Week 11: Animation Integration**
- [ ] Connect animation runtime to render
- [ ] Hover animations
- [ ] Transition on prop change
- [ ] Page transitions

**Week 12: Hot Reload Completion**
- [ ] Connect file watcher to runtime
- [ ] Apply .oui changes live
- [ ] State preservation
- [ ] Error overlay display

### Phase 4: Polish (Weeks 13-16)

**Week 13: Accessibility**
- [ ] Accessibility tree generation
- [ ] Screen reader support (VoiceOver/NVDA)
- [ ] Focus indicators
- [ ] Keyboard-only navigation

**Week 14: Images & Media**
- [ ] Image component
- [ ] File/URL loading
- [ ] Caching
- [ ] Lazy loading

**Week 15: Virtualization**
- [ ] VirtualList component
- [ ] View recycling
- [ ] Infinite scroll
- [ ] Pull to refresh (mobile)

**Week 16: Testing & DevTools**
- [ ] Component testing framework
- [ ] Visual regression setup
- [ ] DevTools UI
- [ ] Performance profiler

---

## Summary

| Category | Tauri | Electron | Flutter | OxideKit |
|----------|-------|----------|---------|----------|
| Events | 100% | 100% | 100% | **0%** |
| Text Input | 100% | 100% | 100% | **0%** |
| IPC | 100% | 100% | 100% | **0%** |
| Windows | 100% | 100% | N/A | **10%** |
| File System | 100% | 100% | Plugin | **0%** |
| Tray/Notifications | 100% | 100% | Plugin | **0%** |
| Menus | 100% | 100% | N/A | **0%** |
| Clipboard | 100% | 100% | Plugin | **0%** |
| Animations | 100% | 100% | 100% | **40%** |
| Accessibility | 90% | 90% | 90% | **5%** |
| Hot Reload | 70% | 80% | 100% | **30%** |
| Design System | 70% | 50% | 80% | **90%** |
| Compile Validation | 20% | 10% | 60% | **80%** |
| Security Model | 80% | 30% | 50% | **70%** |

**Bottom Line:** OxideKit has excellent infrastructure (design system, compile-time validation, security model) but is missing the fundamental runtime features needed to build any interactive application. The 16-week roadmap above is aggressive but achievable.

---

## Recommended Next Steps

1. **Events first** - Without onClick, nothing else matters
2. **Text input second** - Forms are in every app
3. **IPC third** - Connect UI to business logic
4. **File system fourth** - Basic app functionality

Everything else can be plugins or deferred to v0.4.
