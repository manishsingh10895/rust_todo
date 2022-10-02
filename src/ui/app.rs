use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Corner, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
    Frame, Terminal,
};

use crate::models::todo_model::Todo;

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn unselect(&mut self) {
        self.state.select(None);
    }
}

pub enum InputMode {
    None,
    Editing,
}

/// This struct holds the current state of the app. In particular, it has the `items` field which is a wrapper
/// around `ListState`. Keeping track of the items state let us render the associated widget with its state
/// and have access to features such as natural scrolling.
///
/// Check the event handling at the bottom to see how to change the state on incoming events.
/// Check the drawing logic for items on how to specify the highlighting style for selected items.
pub struct App {
    pub undone: StatefulList<Todo>,
    pub done: StatefulList<Todo>,
    pub error_message: String,
    pub input_text: String,
    pub message: String,
    pub input_mode: InputMode,
    navigation_stack: Vec<Route>,
}

#[derive(Debug)]
pub struct Route {
    pub id: RouteId,
    pub active_block: ActiveBlock,
}

#[derive(Clone, PartialEq, Debug)]
pub enum RouteId {
    Home,
    Error,
    Message,
    NewTodo,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ActiveBlock {
    Home,
    Error,
    Message,
    NewTodo,
}

pub const DEFAULT_ROUTE: Route = Route {
    id: RouteId::Home,
    active_block: ActiveBlock::Home,
};

impl App {
    pub fn new() -> App {
        App {
            undone: StatefulList::with_items(vec![]),
            done: StatefulList::with_items(vec![]),
            error_message: String::new(),
            input_mode: InputMode::None,
            input_text: String::new(),
            message: String::new(),
            navigation_stack: vec![DEFAULT_ROUTE],
        }
    }

    /// Rotate through the event list.
    /// This only exists to simulate some kind of "progress"
    pub fn on_tick(&mut self) {}

    /// Gets the current active route
    pub fn get_current_route(&self) -> &Route {
        self.navigation_stack.last().unwrap_or(&DEFAULT_ROUTE)
    }

    pub fn get_current_route_mut(&mut self) -> &mut Route {
        self.navigation_stack.last_mut().unwrap()
    }

    /// Push a route to the navigation stack
    /// so that it is rendered
    pub fn push_navigation_stack(&mut self, route_id: RouteId, active_block: ActiveBlock) {
        self.navigation_stack.push(Route {
            id: route_id,
            active_block,
        });
    }

    pub fn pop_navigation_stack(&mut self) -> Option<Route> {
        if self.navigation_stack.len() == 1 {
            None
        } else {
            self.navigation_stack.pop()
        }
    }

    pub fn handle_error(&mut self, e: String) {
        self.push_navigation_stack(RouteId::Error, ActiveBlock::Error);
        self.error_message = e.to_string();
    }

    pub fn handle_new_message(&mut self, m: String) {
        let active_block = self.navigation_stack.last().unwrap().active_block;
        if active_block == ActiveBlock::Message || active_block == ActiveBlock::Error {
            self.pop_navigation_stack();
        }

        self.push_navigation_stack(RouteId::Message, ActiveBlock::Message);
        self.message = m;
    }
}
