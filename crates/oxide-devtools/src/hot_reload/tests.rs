//! Integration tests for the hot reload system

#[cfg(test)]
mod integration_tests {
    use crate::hot_reload::*;
    use crate::hot_reload::watcher::WatcherConfig;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    // Re-export needed submodules for tests
    use crate::hot_reload::{state, events, overlay, server};

    /// Create a test project structure
    fn create_test_project() -> TempDir {
        let temp_dir = TempDir::new().unwrap();

        // Create oxide.toml
        let manifest = r#"
[app]
id = "dev.test.hotreload"
name = "Hot Reload Test"
version = "0.1.0"

[window]
title = "Test"
width = 800
height = 600

[dev]
hot_reload = true
"#;
        let manifest_path = temp_dir.path().join("oxide.toml");
        let mut file = std::fs::File::create(&manifest_path).unwrap();
        file.write_all(manifest.as_bytes()).unwrap();

        // Create ui directory
        std::fs::create_dir_all(temp_dir.path().join("ui")).unwrap();

        // Create app.oui
        let ui_source = r#"
app TestApp {
    Column {
        align: center
        justify: center

        Text {
            content: "Hello Hot Reload!"
            size: 32
        }
    }
}
"#;
        let ui_path = temp_dir.path().join("ui/app.oui");
        let mut file = std::fs::File::create(&ui_path).unwrap();
        file.write_all(ui_source.as_bytes()).unwrap();

        temp_dir
    }

    /// Create a .oui file with the given content
    fn create_oui_file(dir: &std::path::Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_watcher_config() {
        let config = WatcherConfig::default();
        assert!(config.extensions.contains(&"oui".to_string()));
        assert!(config.extensions.contains(&"rs".to_string()));
        assert!(config.ignore_dirs.contains(&"target".to_string()));
        assert!(config.recursive);
    }

    #[test]
    fn test_state_manager_lifecycle() {
        let manager = StateManager::new();

        // Register a component
        let mut state = state::ComponentState::new("test_1", "Button");
        state.set_state("count", state::StateValue::Number(42.0));
        manager.register_component(state);

        // Capture snapshot
        let snapshot = manager.capture();
        assert_eq!(snapshot.component_count(), 1);

        // Update state
        manager
            .update_component("test_1", |c| {
                c.set_state("count", state::StateValue::Number(100.0));
            })
            .unwrap();

        // Verify update
        let component = manager.get_component("test_1").unwrap();
        assert_eq!(
            component.get_state("count").unwrap().as_number(),
            Some(100.0)
        );

        // Restore original snapshot
        let diff = manager.restore(snapshot).unwrap();
        assert!(!diff.modified.is_empty());
    }

    #[test]
    fn test_incremental_compiler_basic() {
        let temp_dir = TempDir::new().unwrap();
        let source = r#"
app TestApp {
    Text {
        content: "Test"
        size: 24
    }
}
"#;
        let file_path = create_oui_file(temp_dir.path(), "test.oui", source);

        let compiler = IncrementalCompiler::with_defaults();
        let result = compiler.compile_file(&file_path).unwrap();

        assert!(!result.cached);
        assert_eq!(result.ir.kind, "Text");
    }

    #[test]
    fn test_incremental_compiler_caching() {
        let temp_dir = TempDir::new().unwrap();
        let source = r#"
app CacheTest {
    Column {
        Text { content: "A" }
        Text { content: "B" }
    }
}
"#;
        let file_path = create_oui_file(temp_dir.path(), "cache_test.oui", source);

        let compiler = IncrementalCompiler::with_defaults();

        // First compile
        let result1 = compiler.compile_file(&file_path).unwrap();
        assert!(!result1.cached);

        // Second compile should hit cache
        let result2 = compiler.compile_file(&file_path).unwrap();
        assert!(result2.cached);

        // Invalidate and recompile
        compiler.invalidate(&file_path);
        let result3 = compiler.compile_file(&file_path).unwrap();
        assert!(!result3.cached);
    }

    #[test]
    fn test_compile_error_handling() {
        let temp_dir = TempDir::new().unwrap();
        let invalid_source = r#"
app BrokenApp {
    Text {
        content: "Missing closing brace"
    // Missing closing braces
"#;
        let file_path = create_oui_file(temp_dir.path(), "broken.oui", invalid_source);

        let compiler = IncrementalCompiler::with_defaults();
        let result = compiler.compile_file(&file_path);

        assert!(result.is_err());
    }

    #[test]
    fn test_error_overlay() {
        let overlay = ErrorOverlay::with_defaults();

        assert!(!overlay.is_visible());
        assert_eq!(overlay.error_count(), 0);

        // Show errors
        let diagnostic = overlay::DiagnosticDisplay {
            file: PathBuf::from("test.oui"),
            line: 10,
            column: 5,
            message: "Test error message".to_string(),
            severity: events::ErrorSeverity::Error,
            code: Some("E001".to_string()),
            source_snippet: None,
            timestamp: std::time::Instant::now(),
        };

        overlay.show(vec![diagnostic]);
        assert!(overlay.is_visible());
        assert_eq!(overlay.error_count(), 1);

        // Navigation
        let selected = overlay.selected().unwrap();
        assert_eq!(selected.line, 10);

        // Hide
        overlay.hide();
        assert!(!overlay.is_visible());
    }

    #[test]
    fn test_event_bus() {
        let bus = EventBus::with_default_buffer();
        let subscriber = bus.subscribe();

        // Publish events
        bus.publish(HotReloadEvent::CompileStarted {
            files: vec![PathBuf::from("test.oui")],
        });

        bus.publish(HotReloadEvent::CompileSuccess {
            duration_ms: 50,
            changed_components: vec!["App".to_string()],
        });

        // Receive events
        let event1 = subscriber.try_recv().unwrap();
        let event2 = subscriber.try_recv().unwrap();

        match event1 {
            HotReloadEvent::CompileStarted { files } => {
                assert_eq!(files.len(), 1);
            }
            _ => panic!("Wrong event type"),
        }

        match event2 {
            HotReloadEvent::CompileSuccess { duration_ms, .. } => {
                assert_eq!(duration_ms, 50);
            }
            _ => panic!("Wrong event type"),
        }
    }

    #[test]
    fn test_file_change_kind() {
        use std::path::Path;

        assert_eq!(
            events::FileChangeKind::from_path(Path::new("app.oui")),
            events::FileChangeKind::Ui
        );
        assert_eq!(
            events::FileChangeKind::from_path(Path::new("main.rs")),
            events::FileChangeKind::Source
        );
        assert_eq!(
            events::FileChangeKind::from_path(Path::new("oxide.toml")),
            events::FileChangeKind::Config
        );
        assert_eq!(
            events::FileChangeKind::from_path(Path::new("logo.png")),
            events::FileChangeKind::Asset
        );

        // Hot reloadable check
        assert!(events::FileChangeKind::Ui.is_hot_reloadable());
        assert!(events::FileChangeKind::Asset.is_hot_reloadable());
        assert!(!events::FileChangeKind::Source.is_hot_reloadable());
        assert!(!events::FileChangeKind::Config.is_hot_reloadable());
    }

    #[test]
    fn test_source_snippet() {
        let source = "line 1\nline 2\nline 3 with error\nline 4\nline 5";
        let snippet = overlay::SourceSnippet::from_source(source, 3, 10, 1);

        assert_eq!(snippet.lines.len(), 3);
        assert_eq!(snippet.lines[0].number, 2);
        assert_eq!(snippet.lines[1].number, 3);
        assert_eq!(snippet.lines[2].number, 4);
        assert!(snippet.lines[1].is_error_line);
    }

    #[test]
    fn test_state_snapshot_serialization() {
        let manager = StateManager::new();

        let mut state = state::ComponentState::new("input_1", "TextInput");
        state.set_state("value", state::StateValue::String("Hello".to_string()));
        state.scroll_position = Some((0.0, 100.0));
        manager.register_component(state);

        // Export to JSON
        let json = manager.export_json().unwrap();
        assert!(json.contains("input_1"));
        assert!(json.contains("TextInput"));
        assert!(json.contains("Hello"));

        // Import back
        let new_manager = StateManager::new();
        new_manager.import_json(&json).unwrap();

        let component = new_manager.get_component("input_1").unwrap();
        assert_eq!(component.kind, "TextInput");
    }

    #[test]
    fn test_state_diff() {
        let mut old = state::StateSnapshot::new();
        let mut new = state::StateSnapshot::new();

        old.components.insert(
            "a".to_string(),
            state::ComponentState::new("a", "Button"),
        );
        old.components.insert(
            "b".to_string(),
            state::ComponentState::new("b", "Text"),
        );

        new.components.insert(
            "b".to_string(),
            state::ComponentState::new("b", "Text"),
        );
        new.components.insert(
            "c".to_string(),
            state::ComponentState::new("c", "Input"),
        );

        let diff = StateManager::calculate_diff(&old, &new);

        assert!(diff.added.contains(&"c".to_string()));
        assert!(diff.removed.contains(&"a".to_string()));
    }

    #[test]
    fn test_hot_reload_config() {
        let config = HotReloadConfig::default();

        assert!(config.enabled);
        assert!(config.auto_start_server);
        assert!(config.watch_paths.contains(&"ui".to_string()));
        assert!(config.watch_paths.contains(&"src".to_string()));
    }

    #[test]
    fn test_dev_server_config() {
        let config = server::DevServerConfig::default();

        assert_eq!(config.port, DEFAULT_WS_PORT);
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.max_clients, 10);
    }

    #[test]
    fn test_server_message_serialization() {
        let msg = server::ServerMessage::HotReload {
            changed_files: vec!["app.oui".to_string()],
            changed_components: vec!["App".to_string()],
            compile_time_ms: 42,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("hot_reload"));
        assert!(json.contains("app.oui"));
        assert!(json.contains("42"));
    }

    #[test]
    fn test_client_message_deserialization() {
        let json = r#"{"type": "ready", "client_id": "test-123", "capabilities": ["hot_reload", "state_sync"]}"#;
        let msg: server::ClientMessage = serde_json::from_str(json).unwrap();

        match msg {
            server::ClientMessage::Ready { client_id, capabilities } => {
                assert_eq!(client_id, "test-123");
                assert_eq!(capabilities.len(), 2);
            }
            _ => panic!("Wrong message type"),
        }
    }

    #[test]
    fn test_overlay_render_data() {
        let overlay = ErrorOverlay::with_defaults();

        // No data when hidden
        assert!(overlay.render_data().is_none());

        // Show errors
        let diagnostic = overlay::DiagnosticDisplay {
            file: PathBuf::from("test.oui"),
            line: 1,
            column: 1,
            message: "Error".to_string(),
            severity: events::ErrorSeverity::Error,
            code: None,
            source_snippet: None,
            timestamp: std::time::Instant::now(),
        };
        overlay.show(vec![diagnostic]);

        // Now should have render data
        let data = overlay.render_data().unwrap();
        assert_eq!(data.diagnostics.len(), 1);
        assert_eq!(data.error_count, 1);
        assert_eq!(data.opacity, 1.0);
    }

    #[test]
    fn test_component_state_helpers() {
        let mut state = state::ComponentState::new("btn", "Button");

        // Test various state values
        state.set_state("enabled", true.into());
        state.set_state("count", 42.0.into());
        state.set_state("label", "Click me".into());

        assert_eq!(state.get_state("enabled").unwrap().as_bool(), Some(true));
        assert_eq!(state.get_state("count").unwrap().as_number(), Some(42.0));
        assert_eq!(
            state.get_state("label").unwrap().as_string(),
            Some("Click me")
        );
    }

    #[test]
    fn test_cache_stats() {
        let temp_dir = TempDir::new().unwrap();
        let compiler = IncrementalCompiler::with_defaults();

        // Initially empty
        let stats = compiler.cache_stats();
        assert_eq!(stats.entries, 0);

        // Compile a file
        let source = r#"app Test { Text { content: "x" } }"#;
        let file_path = create_oui_file(temp_dir.path(), "stats_test.oui", source);
        let _ = compiler.compile_file(&file_path).unwrap();

        // Should have one entry
        let stats = compiler.cache_stats();
        assert_eq!(stats.entries, 1);

        // Clear cache
        compiler.clear_cache();
        let stats = compiler.cache_stats();
        assert_eq!(stats.entries, 0);
    }
}
