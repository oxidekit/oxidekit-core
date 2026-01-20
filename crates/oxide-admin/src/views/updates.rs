//! Updates view

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};
use oxide_render::Color;
use crate::components::{Alert, AlertProps, AlertVariant, EmptyState, EmptyStateProps};
use crate::state::{AdminState, UpdateCheckStatus};
use crate::state::updates::format_size;
use super::layout::build_page_header;

/// Build the updates view
pub fn build_updates_view(tree: &mut LayoutTree, state: &AdminState) -> NodeId {
    let content_style = StyleBuilder::new()
        .flex_column()
        .width_percent(1.0)
        .gap(24.0)
        .build();

    let mut children = Vec::new();

    // Page header
    let header = build_page_header(
        tree,
        "Updates",
        Some("Check for and install OxideKit updates"),
        &[],
    );
    children.push(header);

    // Current version info
    let version_info = build_version_info(tree, state);
    children.push(version_info);

    // Update status
    if let Some(updates) = &state.updates {
        match updates.status {
            UpdateCheckStatus::Checking => {
                let checking = build_checking_state(tree);
                children.push(checking);
            }
            UpdateCheckStatus::Available => {
                let available = build_available_updates(tree, state);
                children.push(available);
            }
            UpdateCheckStatus::UpToDate => {
                let up_to_date = build_up_to_date(tree);
                children.push(up_to_date);
            }
            UpdateCheckStatus::Error => {
                let error = build_update_error(tree, updates.error.as_deref());
                children.push(error);
            }
            UpdateCheckStatus::Idle => {
                let idle = build_idle_state(tree);
                children.push(idle);
            }
        }
    } else {
        let idle = build_idle_state(tree);
        children.push(idle);
    }

    // Update settings
    let settings = build_update_settings(tree, state);
    children.push(settings);

    tree.new_node_with_children(content_style, &children)
}

fn build_version_info(tree: &mut LayoutTree, state: &AdminState) -> NodeId {
    let card_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .justify_between()
        .width_percent(1.0)
        .padding(24.0)
        .build();

    let card_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#1F2937"))
        .with_border(hex_to_rgba("#374151"), 1.0)
        .with_radius(12.0);

    // Left - version info
    let left_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .gap(16.0)
        .build();

    // Logo
    let logo_style = StyleBuilder::new().size(48.0, 48.0).build();
    let logo_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#3B82F6"))
        .with_radius(12.0);
    let logo = tree.new_visual_node(logo_style, logo_visual);

    // Version text
    let version_style = StyleBuilder::new()
        .flex_column()
        .gap(4.0)
        .build();

    let title_style = StyleBuilder::new().build();
    let title = tree.new_node(title_style);

    let version_text = tree.new_node(StyleBuilder::new().build());

    let version_node = tree.new_node_with_children(version_style, &[title, version_text]);

    let left = tree.new_node_with_children(left_style, &[logo, version_node]);

    // Right - check button
    let button_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .gap(8.0)
        .height(40.0)
        .padding_xy(0.0, 16.0)
        .build();

    let button_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#3B82F6"))
        .with_radius(8.0);

    let button = tree.new_visual_node(button_style, button_visual);

    tree.new_visual_node_with_children(card_style, card_visual, &[left, button])
}

fn build_checking_state(tree: &mut LayoutTree) -> NodeId {
    let container_style = StyleBuilder::new()
        .flex_column()
        .center()
        .width_percent(1.0)
        .padding(48.0)
        .gap(16.0)
        .build();

    // Spinner placeholder
    let spinner_style = StyleBuilder::new().size(48.0, 48.0).build();
    let spinner_visual = NodeVisual::default()
        .with_border(hex_to_rgba("#3B82F6"), 3.0)
        .with_radius(24.0);
    let spinner = tree.new_visual_node(spinner_style, spinner_visual);

    // Text
    let text_style = StyleBuilder::new().build();
    let text = tree.new_node(text_style);

    tree.new_node_with_children(container_style, &[spinner, text])
}

fn build_available_updates(tree: &mut LayoutTree, state: &AdminState) -> NodeId {
    let container_style = StyleBuilder::new()
        .flex_column()
        .width_percent(1.0)
        .gap(16.0)
        .build();

    let mut children = Vec::new();

    // Alert
    let alert = Alert::build(tree, AlertProps {
        title: Some("Updates Available".to_string()),
        message: "New updates are ready to install".to_string(),
        variant: AlertVariant::Info,
        dismissible: false,
        icon: true,
    });
    children.push(alert);

    // Update items
    if let Some(updates) = &state.updates {
        // Core update
        if let Some(core) = &updates.core_update {
            let core_card = build_update_item(
                tree,
                "OxideKit Core",
                &core.current_version,
                &core.new_version,
                core.download_size,
                core.critical,
            );
            children.push(core_card);
        }

        // Plugin updates
        for plugin in &updates.plugin_updates {
            let plugin_card = build_update_item(
                tree,
                &plugin.plugin_name,
                &plugin.current_version,
                &plugin.new_version,
                plugin.update.download_size,
                plugin.update.critical,
            );
            children.push(plugin_card);
        }
    }

    // Install all button
    let install_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .justify_end()
        .width_percent(1.0)
        .build();

    let button_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .gap(8.0)
        .height(44.0)
        .padding_xy(0.0, 20.0)
        .build();

    let button_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#22C55E"))
        .with_radius(8.0);

    let button = tree.new_visual_node(button_style, button_visual);
    let install_row = tree.new_node_with_children(install_style, &[button]);
    children.push(install_row);

    tree.new_node_with_children(container_style, &children)
}

fn build_update_item(
    tree: &mut LayoutTree,
    name: &str,
    current: &str,
    new: &str,
    size: u64,
    critical: bool,
) -> NodeId {
    let item_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .justify_between()
        .width_percent(1.0)
        .padding(16.0)
        .build();

    let item_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#1F2937"))
        .with_border(if critical {
            hex_to_rgba("#EF4444")
        } else {
            hex_to_rgba("#374151")
        }, 1.0)
        .with_radius(12.0);

    // Left - name and versions
    let left_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .gap(16.0)
        .build();

    let icon_style = StyleBuilder::new().size(40.0, 40.0).build();
    let icon_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#3B82F6"))
        .with_radius(8.0);
    let icon = tree.new_visual_node(icon_style, icon_visual);

    let info_style = StyleBuilder::new()
        .flex_column()
        .gap(4.0)
        .build();

    let name_style = StyleBuilder::new().build();
    let name_node = tree.new_node(name_style);

    let version_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .gap(8.0)
        .build();
    let version_node = tree.new_node(version_style);

    let info = tree.new_node_with_children(info_style, &[name_node, version_node]);

    let left = tree.new_node_with_children(left_style, &[icon, info]);

    // Right - size and actions
    let right_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .gap(16.0)
        .build();

    let size_style = StyleBuilder::new().build();
    let size_node = tree.new_node(size_style);

    let install_btn_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .height(36.0)
        .padding_xy(0.0, 16.0)
        .build();

    let install_btn_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#3B82F6"))
        .with_radius(6.0);

    let install_btn = tree.new_visual_node(install_btn_style, install_btn_visual);

    let right = tree.new_node_with_children(right_style, &[size_node, install_btn]);

    tree.new_visual_node_with_children(item_style, item_visual, &[left, right])
}

fn build_up_to_date(tree: &mut LayoutTree) -> NodeId {
    let props = EmptyStateProps::new("You're up to date!")
        .description("OxideKit is running the latest version")
        .icon("check");

    EmptyState::build(tree, props, None)
}

fn build_update_error(tree: &mut LayoutTree, error: Option<&str>) -> NodeId {
    Alert::build(tree, AlertProps {
        title: Some("Update Check Failed".to_string()),
        message: error.unwrap_or("An error occurred while checking for updates").to_string(),
        variant: AlertVariant::Error,
        dismissible: true,
        icon: true,
    })
}

fn build_idle_state(tree: &mut LayoutTree) -> NodeId {
    let props = EmptyStateProps::new("Check for Updates")
        .description("Click the button above to check for available updates")
        .icon("refresh");

    EmptyState::build(tree, props, None)
}

fn build_update_settings(tree: &mut LayoutTree, state: &AdminState) -> NodeId {
    let card_style = StyleBuilder::new()
        .flex_column()
        .width_percent(1.0)
        .padding(24.0)
        .gap(16.0)
        .build();

    let card_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#1F2937"))
        .with_border(hex_to_rgba("#374151"), 1.0)
        .with_radius(12.0);

    // Header
    let header_style = StyleBuilder::new().build();
    let header = tree.new_node(header_style);

    // Settings items
    let settings_style = StyleBuilder::new()
        .flex_column()
        .gap(12.0)
        .build();

    // Auto-check toggle
    let auto_check = build_update_setting_row(
        tree,
        "Automatic Updates",
        "Check for updates automatically",
        state.settings.updates.auto_check,
    );

    // Channel selector
    let channel_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .justify_between()
        .build();

    let channel_label = tree.new_node(StyleBuilder::new().build());
    let channel_select = tree.new_node(StyleBuilder::new().width(150.0).build());

    let channel_row = tree.new_node_with_children(channel_style, &[channel_label, channel_select]);

    let settings = tree.new_node_with_children(settings_style, &[auto_check, channel_row]);

    tree.new_visual_node_with_children(card_style, card_visual, &[header, settings])
}

fn build_update_setting_row(tree: &mut LayoutTree, label: &str, description: &str, value: bool) -> NodeId {
    let row_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .justify_between()
        .build();

    let left_style = StyleBuilder::new()
        .flex_column()
        .gap(2.0)
        .build();

    let label_node = tree.new_node(StyleBuilder::new().build());
    let desc_node = tree.new_node(StyleBuilder::new().build());

    let left = tree.new_node_with_children(left_style, &[label_node, desc_node]);

    // Toggle
    let toggle_style = StyleBuilder::new().size(44.0, 24.0).build();
    let toggle_visual = NodeVisual::default()
        .with_background(if value { hex_to_rgba("#3B82F6") } else { hex_to_rgba("#374151") })
        .with_radius(12.0);
    let toggle = tree.new_visual_node(toggle_style, toggle_visual);

    tree.new_node_with_children(row_style, &[left, toggle])
}

fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex).map(|c| c.to_array()).unwrap_or([1.0, 1.0, 1.0, 1.0])
}
