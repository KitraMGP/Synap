//! Quick capture bar view.

use crate::message::Message;
use crate::state::AppState;
use crate::theme::ThemeColors;
use iced::Length;
use iced::widget::{Container, Row, Text};

/// Render the capture bar
pub fn view(state: &AppState, colors: ThemeColors) -> iced::Element<Message> {
    if !state.capture_bar_visible {
        return Container::new(Text::new(""))
            .height(Length::Fixed(0.0))
            .into();
    }

    Container::new(
        Row::new()
            .spacing(8)
            .push(Text::new("Capture:").color(colors.text))
            .push(Text::new(&state.quick_input).color(colors.text))
            .push(Text::new("Press Enter to save").size(10).color(colors.text_secondary))
    )
    .padding(12)
    .width(Length::Fill)
    .into()
}
