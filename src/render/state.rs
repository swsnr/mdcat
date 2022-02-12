// Copyright 2020 Sebastian Wiesner <sebastian@swsnr.de>

// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::{AnsiStyle, LinkCapability};
use ansi_term::Style;
use std::borrow::Borrow;
use syntect::highlighting::HighlightState;
use syntect::parsing::ParseState;

/// Whether to add a margin.
#[derive(Debug, PartialEq, Copy, Clone)]
pub(super) enum MarginControl {
    /// Always add a margin.
    Margin,
    /// Always add no margin.
    NoMargin,
    /// Add margin unless the current event is a HTML event.
    NoMarginForHtmlOnly,
}

/// State attributes for inline text.
#[derive(Debug, PartialEq, Clone)]
pub struct InlineAttrs {
    /// The style to apply to this piece of inline text.
    pub(super) style: Style,
    /// The indent to add after a line break in inline text.
    pub(super) indent: u16,
}

impl Default for InlineAttrs {
    fn default() -> Self {
        InlineAttrs {
            style: Style::new(),
            indent: 0,
        }
    }
}

impl<T> From<T> for InlineAttrs
where
    T: Borrow<StyledBlockAttrs>,
{
    fn from(attrs: T) -> Self {
        InlineAttrs {
            style: attrs.borrow().style,
            indent: attrs.borrow().indent,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ListItemKind {
    Unordered,
    Ordered(u64),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ListItemState {
    /// The first line after the list bullet/
    StartItem,
    /// Text directly within a list item.
    ItemText,
    /// A list after a nested block.
    ItemBlock,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InlineState {
    /// Inline text.
    ///
    /// Regular inline text without any particular implications.
    InlineText,
    /// Inline link.
    ///
    /// This state suppresses link references being written when reading a link
    /// end event.
    ///
    /// Contains the link capability used to render the link.
    InlineLink(LinkCapability),
    /// A list item.
    ///
    /// This is a hybrid between inline and block state because it can contain nested blocks as well
    /// as immediate inline text.  We define it as inline state mostly for convenience: Since there
    /// are less block than inline elements in Markdown we need to duplicate less state transitions
    /// to deal with the peculiarities of list items.
    ///
    /// List item text carries the type of the current item as well as a "state" which we use for
    /// newline control when ending and starting list items.
    ListItem(ListItemKind, ListItemState),
}

/// State attributes for styled blocks.
#[derive(Debug, PartialEq, Clone)]
pub struct StyledBlockAttrs {
    /// Whether to write a margin before the beginning of a block inside this block.
    pub(super) margin_before: MarginControl,
    /// The indent of this block.
    pub(super) indent: u16,
    /// The general style to apply to children of this block, if possible.
    ///
    /// Note that not all nested blocks inherit style; code blocks for instance will always use
    /// their own dedicated style.
    pub(super) style: Style,
}

impl StyledBlockAttrs {
    pub(super) fn without_margin_before(self) -> Self {
        StyledBlockAttrs {
            margin_before: MarginControl::NoMargin,
            ..self
        }
    }

    pub(super) fn with_margin_before(self) -> Self {
        StyledBlockAttrs {
            margin_before: MarginControl::Margin,
            ..self
        }
    }

    pub(super) fn without_margin_for_html_only(self) -> Self {
        StyledBlockAttrs {
            margin_before: MarginControl::NoMarginForHtmlOnly,
            ..self
        }
    }

    pub(super) fn block_quote(self) -> Self {
        StyledBlockAttrs {
            indent: self.indent + 4,
            style: self.style.italic(),
            ..self
        }
    }
}

impl Default for StyledBlockAttrs {
    fn default() -> Self {
        StyledBlockAttrs {
            margin_before: MarginControl::NoMargin,
            indent: 0,
            style: Style::new(),
        }
    }
}

impl<T> From<T> for StyledBlockAttrs
where
    T: Borrow<InlineAttrs>,
{
    fn from(attrs: T) -> Self {
        StyledBlockAttrs {
            indent: attrs.borrow().indent,
            style: attrs.borrow().style,
            ..StyledBlockAttrs::default()
        }
    }
}

/// Attributes for highlighted blocks, that is, code blocks.
#[derive(Debug, PartialEq)]
pub struct HighlightBlockAttrs {
    pub(super) ansi: AnsiStyle,
    pub(super) parse_state: ParseState,
    pub(super) highlight_state: HighlightState,
    /// The indentation to apply to this code block.
    ///
    /// Code blocks in nested blocks such as quotes, lists, etc. gain an additional indent to align
    /// them in the surrounding block.
    pub(super) indent: u16,
}

#[derive(Debug, PartialEq)]
pub struct LiteralBlockAttrs {
    /// The indent for this block.
    pub(super) indent: u16,
    /// The outer style to include.
    pub(super) style: Style,
}

#[derive(Debug, PartialEq)]
pub enum StackedState {
    /// Styled block.
    ///
    /// A block with attached style.
    StyledBlock(StyledBlockAttrs),
    /// A highlighted block of code.
    HighlightBlock(HighlightBlockAttrs),
    /// A literal block without highlighting.
    LiteralBlock(LiteralBlockAttrs),
    /// A rendered inline image.
    ///
    /// We move to this state when we can render an image directly to the terminal, in order to
    /// suppress intermediate events, namely the image title.
    RenderedImage,
    /// Some inline markup.
    Inline(InlineState, InlineAttrs),
}

impl From<StyledBlockAttrs> for StackedState {
    fn from(attrs: StyledBlockAttrs) -> Self {
        StackedState::StyledBlock(attrs)
    }
}

impl From<HighlightBlockAttrs> for StackedState {
    fn from(attrs: HighlightBlockAttrs) -> Self {
        StackedState::HighlightBlock(attrs)
    }
}

impl From<LiteralBlockAttrs> for StackedState {
    fn from(attrs: LiteralBlockAttrs) -> Self {
        StackedState::LiteralBlock(attrs)
    }
}

/// State attributes for top level.
#[derive(Debug, PartialEq)]
pub struct TopLevelAttrs {
    pub(super) margin_before: MarginControl,
}

impl TopLevelAttrs {
    pub(super) fn margin_before() -> Self {
        TopLevelAttrs {
            margin_before: MarginControl::Margin,
        }
    }

    pub(super) fn no_margin_for_html_only() -> Self {
        TopLevelAttrs {
            margin_before: MarginControl::NoMarginForHtmlOnly,
        }
    }
}

impl Default for TopLevelAttrs {
    fn default() -> Self {
        TopLevelAttrs {
            margin_before: MarginControl::NoMargin,
        }
    }
}

const MAX_STATES: usize = 100;

#[derive(Debug, PartialEq)]
pub struct StateStack {
    /// The top level state this stack grows upon.
    top_level: TopLevelAttrs,
    /// The stack of states
    states: Vec<StackedState>,
}

impl StateStack {
    /// Stack onto the given top level state.
    fn new(top_level: TopLevelAttrs) -> Self {
        StateStack {
            top_level,
            states: Vec::with_capacity(20),
        }
    }

    /// Push a new stacked state.
    ///
    /// Panics if the amount of stacked states is exceeded.
    pub(crate) fn push(mut self, state: StackedState) -> StateStack {
        if MAX_STATES <= self.states.len() {
            panic!(
                "More than {} levels of nesting reached.

Report an issue to https://codeberg.org/flausch/mdcat/issues
including the document causing this panic.",
                MAX_STATES
            )
        }
        self.states.push(state);
        self
    }

    /// Return a state by combining this stack with the current stacked state.
    pub(crate) fn current(self, state: StackedState) -> State {
        State::Stacked(self, state)
    }

    /// Pop a stacked state.
    ///
    /// Returns a stacked state with the last state on the stack and the rest of the stack if the
    /// stack is non-empty, or a toplevel state if the stack is empty.
    pub(crate) fn pop(self) -> State {
        let StateStack {
            mut states,
            top_level,
        } = self;
        match states.pop() {
            None => State::TopLevel(top_level),
            Some(state) => StateStack { top_level, states }.current(state),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum State {
    /// At top level.
    TopLevel(TopLevelAttrs),
    /// A stacked state.
    Stacked(StateStack, StackedState),
}

impl State {
    pub(super) fn stack_onto(top_level: TopLevelAttrs) -> StateStack {
        StateStack::new(top_level)
    }

    pub(super) fn and_data<T>(self, data: T) -> (Self, T) {
        (self, data)
    }
}

impl Default for State {
    fn default() -> Self {
        State::TopLevel(TopLevelAttrs::default())
    }
}

impl From<TopLevelAttrs> for State {
    fn from(attrs: TopLevelAttrs) -> Self {
        State::TopLevel(attrs)
    }
}
