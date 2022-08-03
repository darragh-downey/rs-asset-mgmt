use crossterm::{
    event::{self, Event as CEvent, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};
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


mod db;
// mod fetch;
mod start;
mod zip;


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


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("can run in raw mode");
    // let _data = fetch::fetch();
    // run in background
    // should aim to copy the xml to a local sqlite or json for querying
    start::init().await;
    // need to move this to init
    let _data = zip::fetch();

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
    terminal.clear().or(Err(format!("failed to clear terminal")))?;

    let menu_titles = vec!["Home", "Assets", "Add", "Delete", "Quit", "Fetch"];
    let mut active_menu_item = MenuItem::Home;
    let mut asset_list_state = ListState::default();
    asset_list_state.select(Some(0));

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
                    let asset_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(chunks[1]);

                    let (left, right) = render_assets(&asset_list_state);
                    rect.render_stateful_widget(left, asset_chunks[0], &mut asset_list_state);
                    rect.render_widget(right, asset_chunks[1]);
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
                    db::add_random_asset_to_db().expect("can add new asset");
                }
                KeyCode::Char('d') => {
                    db::remove_asset_at_index(&mut asset_list_state).expect("can remove asset");
                }
                KeyCode::Char('f') => {
                        zip::fetch().await?;
                }
                KeyCode::Down => {
                    if let Some(selected) = asset_list_state.selected() {
                        let amount_assets = db::read_db().expect("can fetch asset list").len();
                        if selected >= amount_assets - 1 {
                            asset_list_state.select(Some(0));
                        } else {
                            asset_list_state.select(Some(selected + 1));
                        }
                    }
                }
                KeyCode::Up => {
                    if let Some(selected) = asset_list_state.selected() {
                        let amount_assets = db::read_db().expect("can fetch asset list").len();
                        if selected > 0 {
                            asset_list_state.select(Some(selected -1));
                        } else {
                            asset_list_state.select(Some(amount_assets - 1));
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
        Spans::from(vec![Span::raw("Press 'v' to view assets, 'a' to add new assets, 'd' to delete the currently selected asset, and 'f' to update vulnerabilities")]),
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


fn render_assets<'a>(asset_list_state: &ListState) -> (List<'a>, Table<'a>) {
    let assets = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Assets")
        .border_type(BorderType::Plain);

    let asset_list = db::read_db().expect("can fetch asset list");
    let items: Vec<_> = asset_list
        .iter()
        .map(|asset| {
            ListItem::new(Spans::from(vec![Span::styled(
                        asset.name.clone(),
                        Style::default(),
            )]))
        })
    .collect();

    let selected_asset = asset_list
        .get(
            asset_list_state
            .selected()
            .expect("there is always a selected asset"),
            )
        .expect("exists")
        .clone();

    let list = List::new(items).block(assets).highlight_style(
        Style::default()
        .bg(Color::Yellow)
        .fg(Color::Black)
        .add_modifier(Modifier::BOLD),
    );

    let asset_detail = Table::new(vec![Row::new(vec![
            Cell::from(Span::raw(selected_asset.id.to_string())),
            Cell::from(Span::raw(selected_asset.name)),
            Cell::from(Span::raw(selected_asset.category)),
            Cell::from(Span::raw(selected_asset.vulnerabilities.to_string())),
            Cell::from(Span::raw(selected_asset.created_at.to_string())),
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

    (list, asset_detail)
}


