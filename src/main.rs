use chrono::prelude::*;
use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use rand::{distributions::Alphanumeric, prelude::*};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
use thiserror::Error;
use tui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{
        Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
    },
    Terminal,
};


const DB_PATH: &str = "./data/db.json";


#[derive(Serialize, Deserialize, Clone)]
struct Asset {
    id: usize,
    name: String,
    category: String,
    vulnerabilities: usize,
    created_at: DateTime<Utc>,
}


#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}


enum Event<I> {
    Input(I),
    Tick,
}


#[derive(Copy, Clone, Debug)]
enum MenuItem {
    Home,
    Assets,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
            MenuItem::Assets => 1,
        }
    }
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("can run in raw mode");

    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear();

    let menu_titles = vec!["Home", "Assets", "Add", "Delete", "Quit"];
    let mut active_menu_item = MenuItem::Home;
    let mut pet_list_state = ListState::default();
    pet_list_state.select(Some(0));

    loop {
        terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                    Constraint::Length(3),
                    Constraint::Min(2),
                    Constraint::Length(3),
                    ].as_ref(),
                )
                .split(size);

            let copyright = Paragraph::new("asset-CLI 2022 - all rights reserved")
                .style(Style::default().fg(Color::LightCyan))
                .alignment(Alignment::Center)
                .block(
                    Block::default()
                    .borders(Borders::ALL)
                    .style(Style::default().fg(Color::White))
                    .title("Copyright")
                    .border_type(BorderType::Plain),
                );

            let menu = menu_titles
                .iter()
                .map(|t| {
                    let (first, rest) = t.split_at(1);
                    Spans::from(vec![
                        Span::styled(
                            first,
                            Style::default()
                            .fg(Color::Yellow)
                            .add_modifier(Modifier::UNDERLINED),
                        ),
                        Span::styled(rest, Style::default().fg(Color::White)),
                    ])
                })
            .collect();

            let tabs = Tabs::new(menu)
                .select(active_menu_item.into())
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(Span::raw("|"));


            rect.render_widget(tabs, chunks[0]);
            match active_menu_item {
                MenuItem::Home => rect.render_widget(render_home(), chunks[1]),
                MenuItem::Assets => {
                    let pet_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(chunks[1]);

                    let (left, right) = render_assets(&pet_list_state);
                    rect.render_stateful_widget(left, pet_chunks[0], &mut pet_list_state);
                    rect.render_widget(right, pet_chunks[1]);
                }
            }

            rect.render_widget(copyright, chunks[2]);
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char('h') => active_menu_item = MenuItem::Home,
                KeyCode::Char('p') => active_menu_item = MenuItem::Assets,
                KeyCode::Char('a') => {
                    add_random_pet_to_db().expect("can add new pet");
                }
                KeyCode::Char('d') => {
                    remove_pet_at_index(&mut pet_list_state).expect("can remove pet");
                }
                KeyCode::Down => {
                    if let Some(selected) = pet_list_state.selected() {
                        let amount_assets = read_db().expect("can fetch pet list").len();
                        if selected >= amount_assets - 1 {
                            pet_list_state.select(Some(0));
                        } else {
                            pet_list_state.select(Some(selected + 1));
                        }
                    }
                }
                KeyCode::Up => {
                    if let Some(selected) = pet_list_state.selected() {
                        let amount_assets = read_db().expect("can fetch pet list").len();
                        if selected > 0 {
                            pet_list_state.select(Some(selected -1));
                        } else {
                            pet_list_state.select(Some(amount_assets - 1));
                        }
                    }
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }

    Ok(())
}




fn render_home<'a>() -> Paragraph<'a> {
    let home = Paragraph::new(vec![
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Welcome")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("to")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::styled(
                "asset-CLI",
                Style::default().fg(Color::LightBlue),
        )]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("Press 'p' to access assets, 'a' to add new assets and 'd' to delete the currently selected pet")]),
        ])
        .alignment(Alignment::Center)
        .block(
            Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Home")
            .border_type(BorderType::Plain),
        );
    home
}


fn render_assets<'a>(pet_list_state: &ListState) -> (List<'a>, Table<'a>) {
    let assets = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Assets")
        .border_type(BorderType::Plain);

    let pet_list = read_db().expect("can fetch pet list");
    let items: Vec<_> = pet_list
        .iter()
        .map(|pet| {
            ListItem::new(Spans::from(vec![Span::styled(
                        pet.name.clone(),
                        Style::default(),
            )]))
        })
    .collect();

    let selected_pet = pet_list
        .get(
            pet_list_state
            .selected()
            .expect("there is always a selected pet"),
            )
        .expect("exists")
        .clone();

    let list = List::new(items).block(assets).highlight_style(
        Style::default()
        .bg(Color::Yellow)
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD),
    );

    let pet_detail = Table::new(vec![Row::new(vec![
            Cell::from(Span::raw(selected_pet.id.to_string())),
            Cell::from(Span::raw(selected_pet.name)),
            Cell::from(Span::raw(selected_pet.category)),
            Cell::from(Span::raw(selected_pet.vulnerabilities.to_string())),
            Cell::from(Span::raw(selected_pet.created_at.to_string())),
    ])])
        .header(Row::new(vec![
                Cell::from(Span::styled(
                        "ID",
                        Style::default().add_modifier(Modifier::BOLD),
                        )),
                Cell::from(Span::styled(
                        "Name",
                        Style::default().add_modifier(Modifier::BOLD),
                        )),
                Cell::from(Span::styled(
                        "Category",
                        Style::default().add_modifier(Modifier::BOLD),
                        )),
                Cell::from(Span::styled(
                        "Vulnerabilities",
                        Style::default().add_modifier(Modifier::BOLD),
                        )),
                Cell::from(Span::styled(
                        "Created At",
                        Style::default().add_modifier(Modifier::BOLD),
                        )),
        ]))
            .block(
                Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::White))
                .title("Detail")
                .border_type(BorderType::Plain),
            )
            .widths(&[
                Constraint::Percentage(5),
                Constraint::Percentage(20),
                Constraint::Percentage(15),
                Constraint::Percentage(10),
                Constraint::Percentage(20),
            ]);

    (list, pet_detail)
}



fn read_db() -> Result<Vec<Asset>, Error> {
    let db_content = fs::read_to_string(DB_PATH)?;
    let parsed: Vec<Asset> = serde_json::from_str(&db_content)?;
    Ok(parsed)
}


fn remove_pet_at_index(pet_list_state: &mut ListState) -> Result<(), Error> {
    if let Some(selected) = pet_list_state.selected() {
        let db_content = fs::read_to_string(DB_PATH)?;
        let mut parsed: Vec<Asset> = serde_json::from_str(&db_content)?;
        parsed.remove(selected);
        fs::write(DB_PATH, &serde_json::to_vec(&parsed)?)?;
        let _amount_assets = read_db().expect("can fetch asset list").len(); // remaining assets you're tracking
        if selected > 0 {
            pet_list_state.select(Some(selected - 1));
        } else {
            pet_list_state.select(Some(0));
        }
    }
    Ok(())
}



fn add_random_pet_to_db() -> Result<Vec<Asset>, Error> {
    let mut rng = rand::thread_rng();
    let db_content = fs::read_to_string(DB_PATH)?;
    let mut parsed: Vec<Asset> = serde_json::from_str(&db_content)?;
    let catsdogs = match rng.gen_range(0, 1) {
        0 => "hardware",
        _ => "software",
    };

    let random_pet = Asset {
        id: rng.gen_range(0, 9999999),
        name: rng.sample_iter(Alphanumeric).take(10).collect(),
        category: catsdogs.to_owned(),
        vulnerabilities: rng.gen_range(1, 15),
        created_at: Utc::now(),
    };
    
    parsed.push(random_pet);
    fs::write(DB_PATH, &serde_json::to_vec(&parsed)?)?;
    Ok(parsed)
}
