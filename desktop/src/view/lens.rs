//! Lens console view.

use crate::message::Message;
use crate::state::AppState;
use crate::theme::ThemeColors;
use iced::Length;
use iced::widget::{Column, Container, Row, Text};

/// Render the lens console
pub fn view(state: &AppState, colors: ThemeColors) -> iced::Element<Message> {
    let all_tags = state.get_all_tags();

    let mut tag_elements = Vec::new();
    for tag in &all_tags {
        tag_elements.push(Text::new(format!("#{}", tag)).size(10).into());
    }

    Container::new(
        Column::new()
            .spacing(4)
            .push(Text::new("Filter by tag:").size(11).color(colors.text_secondary))
            .push(Row::new().spacing(4).extend(tag_elements))
            .push(Text::new(format!("Depth: {}", state.depth_limit)).size(11).color(colors.text_secondary))
            .push(Text::new(format!("{} nodes", state.nodes.len())).size(10).color(colors.text_secondary))
            .push(Text::new(format!("{} connections", state.edges.len())).size(10).color(colors.text_secondary))
    )
    .padding(12)
    .width(Length::Fill)
    .into()
}
