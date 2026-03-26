//! Context stream panel view.

use crate::message::Message;
use crate::state::AppState;
use crate::theme::ThemeColors;
use iced::Length;
use iced::widget::{Column, Container, Row, Text};

/// Render the stream panel
pub fn view(state: &AppState, colors: ThemeColors) -> iced::Element<Message> {
    let Some(node_id) = state.stream_node else {
        return Container::new(Text::new("")).width(Length::Fixed(0.0)).into();
    };

    let Some(node) = state.nodes.get(&node_id) else {
        return Container::new(Text::new("Node not found")).width(Length::Fixed(300.0)).into();
    };

    let mut tag_elements = Vec::new();
    for tag in &node.note.tags {
        tag_elements.push(Text::new(format!("#{}", tag)).size(11).into());
    }

    Container::new(
        Column::new()
            .spacing(8)
            .push(
                Row::new()
                    .spacing(8)
                    .push(Text::new("Context Stream").size(18).color(colors.text))
            )
            .push(Text::new("Content:").size(12).color(colors.text_secondary))
            .push(Text::new(&state.stream_edit_content).size(14).color(colors.text))
            .push(Text::new("Tags:").size(12).color(colors.text_secondary))
            .push(Row::new().spacing(4).extend(tag_elements))
            .push(Text::new(format!("Created: {:?}", node.note.created_at)).size(10).color(colors.text_secondary))
            .push(Text::new(format!("Updated: {:?}", node.note.updated_at)).size(10).color(colors.text_secondary))
    )
    .padding(16)
    .width(Length::Fixed(300.0))
    .into()
}
