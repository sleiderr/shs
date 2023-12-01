use std::{
    io::{self, stdout, Stdout},
    time::{Duration, Instant},
};

use rand::Rng;

use crossterm::{
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{block::Title, *},
};

use crate::signal::SinSignal;

pub mod signal;

fn main() -> io::Result<()> {
    App::run()
}

struct App<'a> {
    decrypt_progress: u16,
    events: Vec<(&'a str, &'a str)>,
    scroll: u16,
    signal: SinSignal,
    data_signal: Vec<(f64, f64)>,
    window: [f64; 2],
}

impl<'a> App<'a> {
    fn new() -> Self {
        let mut signal = SinSignal::new(0.2, 3.0, 18.0);
        let data_signal = signal.by_ref().take(2000).collect::<Vec<(f64, f64)>>();

        Self {
            events: vec![
                ("Message1", "TYPE4"),
                ("Message2", "TYPE4"),
                ("Message3", "TYPE1"),
                ("Message4", "TYPE2"),
                ("Message5", "TYPE4"),
                ("Message6", "TYPE4"),
                ("Message7", "TYPE3"),
                ("Message8", "TYPE4"),
                ("Message9", "TYPE4"),
                ("Message10", "TYPE4"),
                ("Message11", "TYPE1"),
                ("Message12", "TYPE4"),
                ("Message13", "TYPE4"),
                ("Message14", "TYPE4"),
                ("Message15", "TYPE4"),
                ("Message16", "TYPE4"),
                ("Message17", "TYPE2"),
                ("Message18", "TYPE2"),
                ("Message19", "TYPE4"),
                ("Message20", "TYPE4"),
                ("Message21", "TYPE3"),
                ("Message22", "TYPE4"),
                ("Message23", "TYPE4"),
                ("Message24", "TYPE3"),
                ("Message25", "TYPE4"),
                ("Message26", "TYPE4"),
            ],
            decrypt_progress: 0,
            scroll: 0,
            signal,
            data_signal,
            window: [0.0, 20.0],
        }
    }

    pub fn update_decrypt_bar(&mut self) {
        self.decrypt_progress = (self.decrypt_progress + 4).min(100);
    }

    pub fn run() -> io::Result<()> {
        let mut terminal = init_terminal()?;
        let mut app = App::new();
        loop {
            let _ = terminal.draw(|frame| app.ui(frame));
            app.update_decrypt_bar();
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Char('q') => break,
                        _ => {}
                    }
                }
            }
            let event = app.events.remove(0);
            app.events.push(event);

            app.scroll += 1;
            app.scroll %= 10;

            app.data_signal.extend(app.signal.by_ref().take(10));
            app.window[0] += 1.0;
            app.window[1] += 1.0;
        }
        restore_terminal()
    }

    fn ui(&self, frame: &mut Frame) {
        let main_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(frame.size());

        let left_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .split(main_layout[0]);

        let right_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(85), Constraint::Percentage(15)])
            .split(main_layout[1]);

        let create_block = |title| {
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default().fg(Color::Gray))
                .title(Span::styled(
                    title,
                    Style::default().add_modifier(Modifier::BOLD),
                ))
        };

        let x_labels = vec![
            Span::styled(
                format!("{}", self.window[0]),
                Style::default().add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("{}", (self.window[0] + self.window[1]) / 2.0)),
            Span::styled(
                format!("{}", self.window[1]),
                Style::default().add_modifier(Modifier::BOLD),
            ),
        ];

        let datasets = vec![Dataset::default()
            .name("data2")
            .marker(symbols::Marker::Dot)
            .style(Style::default().fg(Color::Cyan))
            .data(&self.data_signal)];

        let chart = Chart::new(datasets)
            .block(
                Block::default()
                    .title("Signal".cyan().bold())
                    .borders(Borders::ALL),
            )
            .x_axis(
                Axis::default()
                    .title("time")
                    .style(Style::default().fg(Color::Gray))
                    .labels(x_labels)
                    .bounds(self.window),
            )
            .y_axis(
                Axis::default()
                    .title("dB")
                    .style(Style::default().fg(Color::Gray))
                    .labels(vec!["-20".bold(), "0".into(), "20".bold()])
                    .bounds([-20.0, 20.0]),
            );
        frame.render_widget(chart, left_layout[0]);

        let mut rng = rand::thread_rng();
        let numbers: Vec<u64> = (0..100).map(|_| rng.gen()).collect();

        let mut text = vec![];

        for number in numbers {
            text.push(Line::from(format!("{:b}", number)));
        }

        let paragraph = Paragraph::new(text)
            .style(Style::default().fg(Color::LightGreen))
            .block(create_block("Center alignment, with wrap, with scroll"))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .scroll((self.scroll, 0));
        frame.render_widget(paragraph, left_layout[1]);

        let events: Vec<ListItem> = self
            .events
            .iter()
            .rev()
            .map(|&(event, level)| {
                let s = match level {
                    "TYPE1" => Style::default().fg(Color::Red),
                    "TYPE2" => Style::default().fg(Color::Magenta),
                    "TYPE3" => Style::default().fg(Color::Yellow),
                    "TYPE4" => Style::default().fg(Color::Blue),
                    _ => Style::default(),
                };
                let header = Line::from(vec![
                    Span::styled(format!("{level:<9}"), s),
                    " ".into(),
                    "2020-01-01 10:00:00".italic(),
                ]);
                let log = Line::from(vec![event.into()]);

                ListItem::new(vec![
                    Line::from("-".repeat(right_layout[0].width as usize)),
                    header,
                    Line::from(""),
                    log,
                ])
            })
            .collect();
        let events_list = List::new(events)
            .block(Block::default().borders(Borders::ALL).title("List"))
            .start_corner(Corner::BottomLeft);
        frame.render_widget(events_list, right_layout[0]);

        let decrypt_progress_bar_title =
            Title::from("Decryption in progress").alignment(Alignment::Center);
        let decrypt_progesss_bar_block = Block::default()
            .borders(Borders::all())
            .title(decrypt_progress_bar_title);
        let decrypt_progress_bar_inner_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ])
            .split(right_layout[1]);
        let decrypt_progress_bar = Gauge::default()
            .gauge_style(Style::new().light_magenta())
            .percent(self.decrypt_progress);

        frame.render_widget(Block::default().borders(Borders::all()), left_layout[1]);
        frame.render_widget(Block::default().borders(Borders::all()), right_layout[0]);
        frame.render_widget(decrypt_progesss_bar_block, right_layout[1]);
        frame.render_widget(decrypt_progress_bar, decrypt_progress_bar_inner_layout[1]);
    }
}

fn init_terminal() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout()))
}

fn restore_terminal() -> io::Result<()> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
