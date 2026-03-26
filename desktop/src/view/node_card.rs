//! Node card view for rendering individual notes on the canvas.

use crate::message::Message;
use crate::state::CanvasNode;
use crate::theme::ThemeColors;
use iced::Length;
use iced::widget::{Column, Container, Row, Text, TextInput};

/// Render a node card (non-editing)
pub fn view(node: &CanvasNode, colors: ThemeColors, focused: bool) -> iced::Element<Message> {
    let border_color = if focused { colors.node_focused } else { colors.node_border };

    let content = if node.note.content.is_empty() {
        Text::new("Empty note")
            .color(colors.text_secondary)
    } else {
        let preview = if node.note.content.len() > 100 {
            format!("{}...", &node.note.content[..100])
        } else {
            node.note.content.clone()
        };
        Text::new(preview)
            .color(colors.text)
    };

    let mut tags = Vec::new();
    for tag in &node.note.tags {
        tags.push(
            Text::new(format!("#{}", tag))
                .size(10)
                .color(colors.secondary)
                .into()
        );
    }

    Container::new(
        Column::new()
            .push(content.size(14))
            .push(
                Row::new().spacing(4).extend(tags)
            )
    )
    .padding(12)
    .width(Length::Fixed(200.0))
    .height(Length::Fixed(100.0))
    .style(move |_: &iced::Theme| -> iced::widget::container::Style {
        iced::widget::container::Style {
            background: Some(iced::Background::Color(colors.node_bg)),
            border: iced::Border {
                color: border_color,
                width: if focused { 2.0 } else { 1.0 },
                radius: iced::border::Radius::from(8),
            },
            ..Default::default()
        }
    })
    .into()
}

/// Render an editing node card with TextInput
pub fn editing_view(node: &CanvasNode, colors: ThemeColors) -> iced::Element<Message> {
    Container::new(
        TextInput::new("", &node.note.content)
            .on_input(|text| Message::UpdateStreamContent(text))
            .padding(8)
    )
    .padding(4)
    .width(Length::Fixed(200.0))
    .height(Length::Fixed(100.0))
    .style(move |_: &iced::Theme| -> iced::widget::container::Style {
        iced::widget::container::Style {
            background: Some(iced::Background::Color(colors.node_bg)),
            border: iced::Border {
                color: colors.node_focused,
                width: 2.0,
                radius: iced::border::Radius::from(8),
            },
            ..Default::default()
        }
    })
    .into()
}
