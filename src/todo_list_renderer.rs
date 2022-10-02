use actix_web::http::header::ContentRange;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use reqwest::{blocking::Response, header::AUTHORIZATION, Method};

use std::{
    error::Error,
    str::FromStr,
    time::{Duration, Instant},
};

use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Corner, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};

use crate::{
    api::errors::TodoApiError,
    errors::{BaseError, TodoError},
    models::todo_model::Todo,
    ui::app::{ActiveBlock, App, InputMode, RouteId},
    utils::get_saved_token,
};

pub fn render_todo_list(todos: Vec<Todo>) -> Result<(), BaseError> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();

    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (undone, done): (Vec<&Todo>, Vec<&Todo>) = todos.iter().partition(|todo| !todo.completed);

    // create app and run it
    let tick_rate = Duration::from_millis(250);
    let mut app = App::new();

    for i in 0..undone.len() {
        app.undone.items.push(undone[i].to_owned());
    }

    for j in 0..done.len() {
        app.done.items.push(done[j].to_owned());
    }

    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;

    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;

    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

/// Handle http request response
fn handle_response(response: Response) -> Result<String, BaseError> {
    match response.status() {
        reqwest::StatusCode::OK => {
            let text = response.text()?;
            return Ok(text);
        }
        reqwest::StatusCode::UNAUTHORIZED | reqwest::StatusCode::FORBIDDEN => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Invalid Response for Signup, UNAUTHORIZED or FORBIDDEN",
            )));
        }
        reqwest::StatusCode::NOT_FOUND => {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Resource Not Found",
            )));
        }
        _ => {
            eprintln!("{:?}", response);
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Invalid Response for Signup",
            )));
        }
    }
}

/// Request To Complete a todo
fn request_complete_todo(todo_id: &str) -> Result<(), BaseError> {
    let request = _make_request("PUT", format!("todo/{}/complete", todo_id).as_str(), None)?;

    let response = reqwest::blocking::Client::new().execute(request)?;

    let _ = handle_response(response);

    Ok(())
}

fn _make_request(
    method: &str,
    short_url: &str,
    data: Option<serde_json::Value>,
) -> anyhow::Result<reqwest::blocking::Request, TodoError> {
    let url = crate::utils::make_api_url(short_url);

    let method = match method.to_uppercase().as_str() {
        "POST" => Some(Method::POST),
        "GET" => Some(Method::GET),
        "DELETE" => Some(Method::DELETE),
        "PUT" => Some(Method::PUT),
        _ => None,
    };

    let mut request = reqwest::blocking::Client::new().request(method.unwrap(), url);

    let token = get_saved_token()?;

    if data.is_some() {
        request = request.json::<serde_json::Value>(&data.unwrap());
    }

    request
        .header(AUTHORIZATION, format!("Bearer {}", token))
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .build()
        .map_err(|e| {
            return e.into();
        })
}

/// Request To Delete a Todo
fn request_delete_todo(todo_id: &str) -> Result<(), BaseError> {
    let request = _make_request("DELETE", format!("todo/{}", todo_id).as_str(), None)?;

    let response = reqwest::blocking::Client::new().execute(request)?;

    let _ = handle_response(response)?;

    Ok(())
}

/// Make a request to add a new todo
fn request_add_todo(title: String) -> Result<Todo, BaseError> {
    let body = serde_json::json!({ "title": title });

    let request = _make_request("POST", "todo", Some(body))?;

    let response = reqwest::blocking::Client::new().execute(request)?;

    let data = handle_response(response)?;

    let value = serde_json::Value::from_str(data.as_str())?;

    let todo: Todo = serde_json::from_value(value)?;

    Ok(todo)
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> std::io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.input_mode {
                    InputMode::None => match key.code {
                        KeyCode::Esc => {
                            //Pop navigation stack if required
                            match app.get_current_route().active_block {
                                ActiveBlock::Error | ActiveBlock::Message => {
                                    app.pop_navigation_stack();
                                }
                                ActiveBlock::NewTodo => {
                                    app.input_text = String::new();
                                    app.pop_navigation_stack();
                                    app.input_mode = InputMode::None;
                                }
                                _ => {}
                            }
                        }
                        KeyCode::Char('q') => return Ok(()),
                        KeyCode::Left => app.undone.unselect(),
                        KeyCode::Down => app.undone.next(),
                        KeyCode::Up => app.undone.previous(),
                        KeyCode::Char('a') => match app.get_current_route().active_block {
                            ActiveBlock::Home => {
                                app.push_navigation_stack(RouteId::NewTodo, ActiveBlock::NewTodo);
                                app.input_mode = InputMode::Editing;
                                app.input_text = String::new();
                            }
                            _ => {}
                        },
                        KeyCode::Char('d') => {
                            let selected_index = app.undone.state.selected().unwrap();

                            let selected_item = &app.undone.items[selected_index];

                            match request_complete_todo(selected_item.id.to_string().as_str()) {
                                Ok(_) => {
                                    if let Some(item) = app.undone.items.get_mut(selected_index) {
                                        app.done.items.push(item.to_owned());
                                    } else {
                                        app.handle_error(format!("No item at {}", selected_index));
                                    }
                                    app.undone.items.remove(selected_index);
                                }
                                Err(e) => app.handle_error(e.to_string()),
                            }
                        }
                        KeyCode::Char('x') => {
                            let selected_index = app.undone.state.selected().unwrap();

                            let selected_item = &app.undone.items[selected_index];

                            let id = selected_item.id.to_string();

                            match request_delete_todo(id.as_str()) {
                                Ok(_) => {
                                    app.undone.items.remove(selected_index);
                                    app.handle_new_message(String::from("Todo Item Deleted"));
                                }
                                Err(e) => {
                                    app.handle_error(format!(
                                        "Error for resource , {}, \n e => {}",
                                        id,
                                        e.to_string()
                                    ));
                                }
                            }
                        }
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Char(c) => {
                            app.input_text.push(c);
                        }
                        KeyCode::Backspace => {
                            app.input_text.pop();
                        }
                        KeyCode::Esc => {
                            app.pop_navigation_stack();
                            app.input_mode = InputMode::None;
                            app.input_text = String::new();
                        }
                        KeyCode::Enter => {
                            // Create new Todo
                            println!("TODO ITEM {}", app.input_text);
                            app.input_mode = InputMode::None;
                            let todo_title = app.input_text;

                            app.input_text = String::new();
                            app.pop_navigation_stack();

                            match request_add_todo(todo_title) {
                                Ok(todo) => {
                                    let new_todo = Todo {
                                        id: todo.id,
                                        title: todo.title,
                                        created_at: todo.created_at,
                                        completed: todo.completed,
                                        updated_at: todo.updated_at,
                                        user_id: todo.user_id,
                                    };

                                    app.undone.items.insert(0, new_todo.clone());
                                }
                                Err(e) => {
                                    app.handle_error(e.to_string());
                                }
                            }
                        }
                        _ => {}
                    },
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

// Draws Message if occured
fn draw_message_content<B>(f: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(5)
        .split(f.size());

    let message_text = vec![Spans::from(vec![
        Span::raw("New Message: "),
        Span::styled(&app.message, Style::default().fg(Color::LightBlue)),
    ])];

    let message_paragraph = Paragraph::new(message_text)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(Span::styled(
                    "Message",
                    Style::default().fg(Color::LightBlue),
                ))
                .border_style(Style::default().fg(Color::LightCyan)),
        );

    f.render_widget(message_paragraph, chunks[0]);
}

fn draw_new_todo_content<B>(f: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .margin(5)
        .split(f.size());

    let prompt_message = vec![
        Span::raw("Enter New Todo title"),
        Span::raw("Press "),
        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to stop editing, "),
        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to add the todo item"),
    ];

    let help_text = Text::from(Spans::from(prompt_message));
    let help_para = Paragraph::new(help_text);

    f.render_widget(help_para, chunks[0]);

    let input = Paragraph::new(app.input_text.as_ref())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Input"));

    f.render_widget(input, chunks[1]);
}

// Draws Error if occured
fn draw_error_content<B>(f: &mut Frame<B>, app: &App)
where
    B: Backend,
{
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .margin(5)
        .split(f.size());

    let error_text = vec![Spans::from(vec![
        Span::raw("Error Occured: "),
        Span::styled(&app.error_message, Style::default().fg(Color::Red)),
    ])];

    let error_paragraph = Paragraph::new(error_text).wrap(Wrap { trim: true }).block(
        Block::default()
            .borders(Borders::ALL)
            .title(Span::styled("Error", Style::default().fg(Color::Red)))
            .border_style(Style::default().fg(Color::LightRed)),
    );

    f.render_widget(error_paragraph, chunks[0]);
}

fn draw_home_content<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    // Create two chunks with equal horizontal screen space
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(f.size());

    // Iterate through all elements in the `items` app and append some debug text to it.
    let items: Vec<ListItem> = app
        .undone
        .items
        .iter()
        .map(|todo| {
            let lines = vec![Spans::from(todo.title.as_str())];
            ListItem::new(lines).style(Style::default().fg(Color::Black).bg(Color::White))
        })
        .collect();

    // Create a List from all list items and highlight the currently selected one
    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("Todo"))
        .highlight_style(
            Style::default()
                .bg(Color::LightYellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");

    // We can now render the item list
    f.render_stateful_widget(items, chunks[0], &mut app.undone.state);

    // Let's do the same for the events.
    // The event list doesn't have any state and only displays the current state of the list.
    let events: Vec<ListItem> = app
        .done
        .items
        .iter()
        .rev()
        .map(|todo| {
            // Colorcode the level depending on its type
            // Add a example datetime and apply proper spacing between them
            let s = Style::default();
            let header = Spans::from(vec![
                Span::styled(format!("{:<9}", "Completed At"), s),
                Span::raw(" "),
                Span::styled(
                    todo.updated_at.to_string(),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
            ]);
            // The event gets its own line
            let log = Spans::from(vec![Span::styled(
                todo.title.as_str(),
                Style::default().add_modifier(Modifier::BOLD),
            )]);

            // Here several things happen:
            // 1. Add a `---` spacing line above the final list entry
            // 2. Add the Level + datetime
            // 3. Add a spacer line
            // 4. Add the actual event
            ListItem::new(vec![
                Spans::from("-".repeat(chunks[1].width as usize)),
                header,
                Spans::from(""),
                log,
            ])
        })
        .collect();

    let events_list = List::new(events)
        .block(Block::default().borders(Borders::ALL).title("Done"))
        .start_corner(Corner::BottomLeft);

    f.render_widget(events_list, chunks[1]);
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let current_route = app.get_current_route();

    match current_route.active_block {
        ActiveBlock::Message => draw_message_content(f, app),
        ActiveBlock::Home => draw_home_content(f, app),
        ActiveBlock::Error => draw_error_content(f, app),
        ActiveBlock::NewTodo => draw_new_todo_content(f, app),
    }
}
