#![warn(missing_docs)]

mod changes;
mod chat_details;
mod command;
mod commit_details;
mod commitlist;
mod cred;
mod diff;
mod revision_files;
mod status_tree;
mod syntax_text;
mod textinput;
mod topiclist;
mod utils;

use anyhow::Result;
pub use changes::ChangesComponent;
pub use chat_details::ChatDetailsComponent;
pub use command::{CommandInfo, CommandText};
pub use commit_details::CommitDetailsComponent;
pub use commitlist::CommitList;
pub use cred::CredComponent;
use crossterm::event::Event;
pub use diff::DiffComponent;
use ratatui::{
    layout::{Alignment, Rect},
    text::{Span, Text},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
pub use revision_files::RevisionFilesComponent;
pub use syntax_text::SyntaxTextComponent;
pub use textinput::{InputType, TextInputComponent};
pub use topiclist::TopicList;
pub use utils::{
    filetree::FileTreeItemKind, logitems::ItemBatch, scroll_vertical::VerticalScroll,
    string_width_align, time_to_string,
};

pub use self::status_tree::StatusTreeComponent;
use crate::ui::style::Theme;

/// creates accessors for a list of components
/// allows generating code to make sure
/// we always enumerate all components in both getter functions
#[macro_export]
macro_rules! accessors {
    ($self:ident, [$($element:ident),+]) => {
        fn components(& $self) -> Vec<&dyn Component> {
            vec![
                $(&$self.$element,)+
            ]
        }

        fn components_mut(&mut $self) -> Vec<&mut dyn Component> {
            vec![
                $(&mut $self.$element,)+
            ]
        }
    };
}

/// creates a function to determine if any popup is visible
#[macro_export]
macro_rules! any_popup_visible {
    ($self:ident, [$($element:ident),+]) => {
        fn any_popup_visible(& $self) -> bool{
            ($($self.$element.is_visible()) || +)
        }
    };
}

/// creates the draw popup function
#[macro_export]
macro_rules! draw_popups {
    ($self:ident, [$($element:ident),+]) => {
        fn draw_popups(& $self, mut f: &mut Frame) -> Result<()>{
            //TODO: move the layout part out and feed it into `draw_popups`
            let size = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Min(1),
                    Constraint::Length($self.cmdbar.borrow().height()),
                ]
                .as_ref(),
            )
                          .split(f.area())[0];
            ($($self.$element.draw(&mut f, size)?) , +);

            return Ok(());
        }
    };
}

/// simply calls
/// `any_popup_visible`!() and `draw_popups`!() macros
#[macro_export]
macro_rules! setup_popups {
    ($self:ident, [$($element:ident),+]) => {
        $crate::any_popup_visible!($self, [$($element),+]);
        $crate::draw_popups!($self, [ $($element),+ ]);
    };
}

/// returns `true` if event was consumed
pub fn event_pump(ev: &Event, components: &mut [&mut dyn Component]) -> Result<EventState> {
    for c in components {
        if c.event(ev)?.is_consumed() {
            return Ok(EventState::Consumed);
        }
    }

    Ok(EventState::NotConsumed)
}

/// helper fn to simplify delegating command
/// gathering down into child components
/// see `event_pump`,`accessors`
pub fn command_pump(out: &mut Vec<CommandInfo>, force_all: bool, components: &[&dyn Component]) {
    for c in components {
        if c.commands(out, force_all) != CommandBlocking::PassingOn && !force_all {
            break;
        }
    }
}

/// ScrollType
#[derive(Copy, Clone)]
pub enum ScrollType {
    /// Up
    Up,
    /// Down
    Down,
    /// Home
    Home,
    /// End
    End,
    /// PageUp
    PageUp,
    /// PageDown
    PageDown,
}

/// HorizontalScrollType
#[derive(Copy, Clone)]
pub enum HorizontalScrollType {
    /// Left
    Left,
    /// Right
    Right,
}

/// Direction
#[derive(Copy, Clone)]
pub enum Direction {
    /// Up
    Up,
    /// Down
    Down,
}

/// CommandBlocking
#[derive(PartialEq, Eq)]
pub enum CommandBlocking {
    /// Blocking
    Blocking,
    /// PassingOn
    PassingOn,
}

/// visibility_blocking
pub fn visibility_blocking<T: Component>(comp: &T) -> CommandBlocking {
    if comp.is_visible() {
        CommandBlocking::Blocking
    } else {
        CommandBlocking::PassingOn
    }
}

/// DrawableComponent
pub trait DrawableComponent {
    /// draw
    fn draw(&self, f: &mut Frame, rect: Rect) -> Result<()>;
}

/// EventState
#[derive(PartialEq, Eq)]
pub enum EventState {
    /// Consumed
    Consumed,
    /// notConsumed
    NotConsumed,
}

/// FuzzyFinderTarget
#[derive(Copy, Clone, Debug)]
pub enum FuzzyFinderTarget {
    /// Branches
    Branches,
    /// Files
    Files,
    //Home,
}

impl EventState {
    /// is_consumed
    pub fn is_consumed(&self) -> bool {
        *self == Self::Consumed
    }
}

impl From<bool> for EventState {
    fn from(consumed: bool) -> Self {
        if consumed {
            Self::Consumed
        } else {
            Self::NotConsumed
        }
    }
}

/// base component trait
pub trait Component {
    /// command
    fn commands(&self, out: &mut Vec<CommandInfo>, force_all: bool) -> CommandBlocking;

    /// event
    fn event(&mut self, ev: &Event) -> Result<EventState>;

    /// focused
    fn focused(&self) -> bool {
        false
    }
    /// focus/unfocus this component depending on param
    fn focus(&mut self, _focus: bool) {}
    /// is_visible
    fn is_visible(&self) -> bool {
        true
    }
    /// hide
    fn hide(&mut self) {}
    /// show
    fn show(&mut self) -> Result<()> {
        Ok(())
    }

    /// toggle_visible
    fn toggle_visible(&mut self) -> Result<()> {
        if self.is_visible() {
            self.hide();
            Ok(())
        } else {
            self.show()
        }
    }
}

fn dialog_paragraph<'a>(
    title: &'a str,
    content: Text<'a>,
    theme: &Theme,
    focused: bool,
) -> Paragraph<'a> {
    Paragraph::new(content)
        .block(
            Block::default()
                .title(Span::styled(title, theme.title(focused)))
                .borders(Borders::ALL)
                .border_style(theme.block(focused)),
        )
        .alignment(Alignment::Left)
}
