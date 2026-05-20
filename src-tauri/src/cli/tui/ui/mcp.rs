use super::*;

pub(super) fn mcp_rows_filtered<'a>(app: &App, data: &'a UiData) -> Vec<McpDisplayRow<'a>> {
    let query = app.filter.query_lower();
    data.mcp
        .display_rows()
        .into_iter()
        .filter(|row| match &query {
            None => true,
            Some(q) => row.name().to_lowercase().contains(q) || row.id().to_lowercase().contains(q),
        })
        .collect()
}

pub(super) fn render_mcp(
    frame: &mut Frame<'_>,
    app: &App,
    data: &UiData,
    area: Rect,
    theme: &super::theme::Theme,
) {
    let visible = mcp_rows_filtered(app, data);

    let header = Row::new(vec![
        Cell::from(texts::header_name()),
        centered_cell(texts::tui_mcp_live_header()),
        centered_cell("Claude"),
        centered_cell("Codex"),
        centered_cell("Gemini"),
        centered_cell("OpenCode"),
        centered_cell("OpenClaw"),
        centered_cell("Hermes"),
    ])
    .style(Style::default().fg(theme.dim).add_modifier(Modifier::BOLD));

    let rows = visible.iter().map(|row| {
        let live_marker = mcp_live_marker(row.drift_kind(&data.mcp));
        let name = row
            .live_spec_summary()
            .map(|summary| format!("{} {}", row.name(), summary))
            .unwrap_or_else(|| row.name().to_string());
        Row::new(vec![
            Cell::from(name),
            centered_cell(live_marker),
            centered_cell(if row.app_enabled(&AppType::Claude) {
                texts::tui_marker_active()
            } else {
                texts::tui_marker_inactive()
            }),
            centered_cell(if row.app_enabled(&AppType::Codex) {
                texts::tui_marker_active()
            } else {
                texts::tui_marker_inactive()
            }),
            centered_cell(if row.app_enabled(&AppType::Gemini) {
                texts::tui_marker_active()
            } else {
                texts::tui_marker_inactive()
            }),
            centered_cell(if row.app_enabled(&AppType::OpenCode) {
                texts::tui_marker_active()
            } else {
                texts::tui_marker_inactive()
            }),
            centered_cell(if row.app_enabled(&AppType::OpenClaw) {
                texts::tui_marker_active()
            } else {
                texts::tui_marker_inactive()
            }),
            centered_cell(if row.app_enabled(&AppType::Hermes) {
                texts::tui_marker_active()
            } else {
                texts::tui_marker_inactive()
            }),
        ])
    });

    let outer = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain)
        .border_style(pane_border_style(app, Focus::Content, theme))
        .title(texts::menu_manage_mcp());
    frame.render_widget(outer.clone(), area);
    let inner = outer.inner(area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(3),
            Constraint::Min(0),
        ])
        .split(inner);

    if app.focus == Focus::Content {
        render_key_bar_center(
            frame,
            chunks[0],
            theme,
            &[
                ("x", texts::tui_key_toggle()),
                ("m", texts::tui_key_apps()),
                ("a", texts::tui_key_add()),
                ("e", texts::tui_key_edit()),
                ("i", texts::tui_mcp_action_import_existing()),
                ("r", texts::tui_key_resolve()),
                ("d", texts::tui_key_delete()),
            ],
        );
    }

    let mut summary = texts::tui_mcp_server_counts(
        data.mcp
            .rows
            .iter()
            .filter(|row| row.server.apps.claude)
            .count(),
        data.mcp
            .rows
            .iter()
            .filter(|row| row.server.apps.codex)
            .count(),
        data.mcp
            .rows
            .iter()
            .filter(|row| row.server.apps.gemini)
            .count(),
        data.mcp
            .rows
            .iter()
            .filter(|row| row.server.apps.opencode)
            .count(),
        data.mcp
            .rows
            .iter()
            .filter(|row| row.server.apps.openclaw)
            .count(),
        data.mcp
            .rows
            .iter()
            .filter(|row| row.server.apps.hermes)
            .count(),
    );
    let drift_counts = data.mcp.live_drift_counts();
    if drift_counts.has_drift() {
        summary = format!(
            "{} · {summary}",
            texts::tui_mcp_live_drift_summary(
                drift_counts.changed,
                drift_counts.live_only,
                drift_counts.db_only,
                drift_counts.invalid,
            )
        );
    }
    render_summary_bar(frame, chunks[1], theme, summary);

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(34),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(9),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::NONE))
    .row_highlight_style(selection_style(theme))
    .highlight_symbol(highlight_symbol(theme));

    let mut state = TableState::default();
    state.select(Some(app.mcp_idx));

    frame.render_stateful_widget(table, inset_left(chunks[2], CONTENT_INSET_LEFT), &mut state);
}

fn centered_cell(text: impl Into<String>) -> Cell<'static> {
    Cell::from(Line::from(text.into()).alignment(Alignment::Center))
}

fn mcp_live_marker(kind: Option<&crate::services::McpLiveDriftKind>) -> &'static str {
    match kind {
        Some(crate::services::McpLiveDriftKind::Changed) => "~",
        Some(crate::services::McpLiveDriftKind::LiveOnly) => "+",
        Some(crate::services::McpLiveDriftKind::DbOnly) => "-",
        Some(crate::services::McpLiveDriftKind::LiveInvalid) => "!",
        _ => "",
    }
}
