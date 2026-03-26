//! Infinite canvas view for the knowledge graph.

use crate::message::Message;
use crate::state::AppState;
use crate::theme::ThemeColors;
use crate::view::node_card;
use iced::Length;
use iced::widget::{Column, Container, Row, Text, Button};

/// Render the infinite canvas
pub fn view(state: &AppState, colors: ThemeColors) -> iced::Element<'_, Message> {
    if state.nodes.is_empty() {
        return Container::new(
            Column::new()
                .spacing(16)
                .width(Length::Fill)
                .align_x(iced::alignment::Horizontal::Center)
                .push(
                    Text::new("No notes yet")
                        .size(24)
                        .color(colors.text_secondary)
                )
                .push(
                    Text::new("Use the capture bar above to create your first note")
                        .size(14)
                        .color(colors.text_secondary)
                )
                .push(
                    Button::new(Text::new("Create sample note"))
                        .on_press(Message::QuickCapture("This is my first thought!".to_string()))
                )
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into();
    }

    let mut node_elements = Vec::new();
    for (id, node) in &state.nodes {
        let is_focused = state.focused_node == Some(*id);
        let node_card = node_card::view(node, colors.clone(), is_focused);
        node_elements.push(node_card);
    }

    Container::new(
        Column::new()
            .push(
                Text::new(format!("{} nodes", state.nodes.len()))
                    .size(12)
                    .color(colors.text_secondary)
            )
            .push(
                Row::new()
                    .spacing(12)
                    .extend(node_elements)
            )
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(20)
    .into()
}
