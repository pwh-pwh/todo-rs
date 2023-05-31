use ncurses::CURSOR_VISIBILITY::CURSOR_INVISIBLE;
use ncurses::*;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process::exit;
use std::{env, process};

const REGULAR_PAIR: i16 = 0;
const HIGHLIGHT_PAIR: i16 = 1;

type Id = usize;

#[derive(Debug)]
enum Status {
    Todo,
    Done,
}
impl Status {
    fn toggle(self) -> Self {
        match self {
            Status::Done => Status::Todo,
            Status::Todo => Status::Done,
        }
    }
}

fn parse_item(line: &str) -> Option<(Status, &str)> {
    let todo_prefix = "TODO: ";
    let done_prefix = "DONE: ";
    if line.starts_with(todo_prefix) {
        return Some((Status::Todo, &line[todo_prefix.len()..]));
    }
    if line.starts_with(done_prefix) {
        return Some((Status::Done, &line[done_prefix.len()..]));
    }
    None
}

fn save_state(todos: &Vec<String>, dones: &Vec<String>, file_path: &str) {
    let mut file = File::create(file_path).unwrap();
    for todo in todos.iter() {
        writeln!(file, "TODO: {}", todo);
    }
    for done in dones.iter() {
        writeln!(file, "DONE: {}", done);
    }
}

fn load_state(todos: &mut Vec<String>, dones: &mut Vec<String>, file_path: &str) {
    let file = File::open(file_path).unwrap();
    for (index, line) in BufReader::new(file).lines().enumerate() {
        match parse_item(&line.unwrap()) {
            Some((Status::Todo, title)) => todos.push(title.into()),
            Some((Status::Done, title)) => dones.push(title.into()),
            _ => {
                eprintln!("{file_path} {index} ill-format", index = index + 1);
                process::exit(1);
            }
        }
    }
}

#[derive(Default)]
struct Ui {
    list_cur: Option<Id>,
    row: usize,
    col: usize,
}
impl Ui {
    fn begin(&mut self, row: usize, col: usize) {
        self.row = row;
        self.col = col;
    }
    fn end(&mut self) {}
    fn label(&mut self, label: &str, pair: i16) {
        mv(self.row as i32, self.col as i32);
        attron(COLOR_PAIR(pair));
        addstr(label);
        attroff(COLOR_PAIR(pair));
        self.row += 1;
    }
    fn begin_list(&mut self, id: Id) {
        assert!(self.list_cur.is_none(), "nested list not allow");
        self.list_cur = Some(id);
    }
    fn list_element(&mut self, label: &str, id: Id) {
        let id_curr = self.list_cur.expect("not allow");
        let pair = if id_curr == id {
            HIGHLIGHT_PAIR
        } else {
            REGULAR_PAIR
        };
        self.label(label, pair);
    }

    fn end_list(&mut self) {
        self.list_cur = None;
    }
}

fn list_up(list_cur: &mut usize) {
    if *list_cur > 0 {
        *list_cur -= 1;
    }
}
fn list_down(list: &Vec<String>, list_cur: &mut usize) {
    if *list_cur + 1 < list.len() {
        *list_cur += 1;
    }
}

fn list_transfer(list_dst: &mut Vec<String>, list_src: &mut Vec<String>, list_src_cur: &mut usize) {
    if *list_src_cur < list_src.len() {
        list_dst.push(list_src.remove(*list_src_cur));
        if *list_src_cur >= list_src.len() && list_src.len() > 0 {
            *list_src_cur = list_src.len() - 1;
        }
    }
}

fn main() {
    let mut args = env::args();
    args.next().unwrap();
    let file_path = args.next().expect("should input file_path");

    let mut todos = Vec::<String>::new();
    let mut todo_cur = 0usize;
    let mut dones = Vec::<String>::new();
    let mut done_cur = 0usize;

    load_state(&mut todos, &mut dones, &file_path);

    initscr();
    start_color();
    curs_set(CURSOR_INVISIBLE);
    noecho();
    init_pair(REGULAR_PAIR, COLOR_WHITE, COLOR_BLACK);
    init_pair(HIGHLIGHT_PAIR, COLOR_BLACK, COLOR_WHITE);
    let mut quit = false;

    let mut ui = Ui::default();
    let mut tab = Status::Todo;
    while !quit {
        erase();
        ui.begin(0, 0);
        match tab {
            Status::Todo => {
                ui.label("[TODO] DONE", REGULAR_PAIR);
                ui.label("------------", REGULAR_PAIR);
                ui.begin_list(todo_cur);
                for (index, todo) in todos.iter().enumerate() {
                    ui.list_element(&format!("- [ ] {}", todo), index);
                }
                ui.end_list();
            }
            Status::Done => {
                ui.label(" TODO [DONE]", REGULAR_PAIR);
                ui.label("------------", REGULAR_PAIR);
                ui.begin_list(done_cur);
                for (index, done) in dones.iter().enumerate() {
                    ui.list_element(&format!("- [x] {}", done), index);
                }
                ui.end_list();
            }
        }

        ui.end();
        refresh();
        let key = getch();
        match key as u8 as char {
            'q' => quit = true,
            'w' => match tab {
                Status::Todo => list_up(&mut todo_cur),
                Status::Done => list_up(&mut done_cur),
            },
            's' => match tab {
                Status::Todo => list_down(&todos, &mut todo_cur),
                Status::Done => list_down(&dones, &mut done_cur),
            },
            '\n' => match tab {
                Status::Todo => {
                    list_transfer(&mut dones, &mut todos, &mut todo_cur);
                }
                Status::Done => {
                    list_transfer(&mut todos, &mut dones, &mut done_cur);
                }
            },
            '\t' => {
                tab = tab.toggle();
            }
            _ => {}
        }
    }
    save_state(&todos, &dones, &file_path);
    endwin();
}
