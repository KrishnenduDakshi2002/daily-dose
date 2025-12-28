use core::task;
use std::{collections::HashMap, fs, str::FromStr};

use chrono::{DateTime, Datelike, Local, NaiveDate, Utc};
use clap::{arg, builder, command, error::ErrorKind, value_parser, ArgAction, ArgMatches, Command};
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use rusqlite::{
    types::{FromSql, ToSqlOutput},
    Connection, Error, ToSql,
};
use strum::{Display, EnumString};
use ulid::Ulid;

#[derive(Display, EnumString, Debug)]
#[strum(serialize_all = "snake_case")]
enum Status {
    Todo,
    InProgress,
    Done,
    Blocked,
}

#[derive(Debug)]
struct Task {
    id: String,
    description: String,
    status: Status,
    date: String,
}

#[derive(Debug)]
struct GroupedTasks {
    date: String,
    tasks: Vec<Task>,
}

impl ToSql for Status {
    /*
     * i received 'trait bound not satisfied error'
     * then in the rusqlite documentation searched for the asked trait
     * found ToSql trait and it's various implementations
     * checked the 'impl ToSql for String' as it look closed to the above enum
     * then visited the source to check how they have implemented that, so i can do the same for
     * this enum
     *
     * https://docs.rs/rusqlite/latest/src/rusqlite/types/to_sql.rs.html#257-262
     * */
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.to_string()))
    }
}

impl FromSql for Status {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        value.as_str().map(|s| Status::from_str(s).unwrap())
    }
}

fn get_db_path() -> String {
    let mut data_dir = dirs::data_dir().expect("Could not find data directory in OS");

    data_dir.push("daily-dose");

    fs::create_dir_all(&data_dir).expect("Failed to create directory");

    data_dir.push("storage.db");

    let db_path = data_dir.to_str().expect("Path conversion failed to str");

    db_path.to_string()
}

fn open_db_connection() -> Result<Connection, Error> {
    let path = get_db_path();
    let connection = Connection::open(path)?;
    Ok(connection)
}

fn create_task_table(conn: &Connection) -> Result<(), Error> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS tasks (
            id   TEXT PRIMARY KEY,
            description TEXT NOT NULL,
            status TEXT NOT NULL,
            date TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        (), // empty list of parameters.
    )?;

    Ok(())
}

fn construct_timestamp(arg_matches: &ArgMatches) -> NaiveDate {
    let mut timestamp = Local::now().date_naive();
    /*
     * reason of this year to day approach is only for day case
     * as no. of day will depend on the month
     *
     * Assume current date is some day (15th) in april month (support till 30th)
     * if user passes -d 31 -m 3 (31st march) -> valid
     *
     * in this case if we first modify day then will be modifing it to 31st april which is
     * invalid.
     * That's why we are first modifying month (if mentioned) so we will first convert it to
     * 15th march, then we will convert to 31st march
     *
     * */

    if let Some(year) = arg_matches.get_one::<u32>("year") {
        println!("Year no = {}", year);
        match timestamp.with_year(year.to_owned() as i32) {
            Some(date) => timestamp = date,
            None => {
                // invalid date
                panic!("Invalid date");
            }
        }
    }

    if let Some(month) = arg_matches.get_one::<u32>("month") {
        println!("Month no = {}", month);
        match timestamp.with_month(month.to_owned()) {
            Some(date) => timestamp = date,
            None => {
                // invalid date
                panic!("Invalid date");
            }
        }
    }

    if let Some(day) = arg_matches.get_one::<u32>("day") {
        println!("Day no = {}", day);
        match timestamp.with_day(day.to_owned()) {
            Some(date) => timestamp = date,
            None => {
                // invalid date
                panic!("Invalid date");
            }
        }
    }

    timestamp
}

fn display_format_timestamp(timestamp: &NaiveDate) -> String {
    format!("{}", timestamp.format("%d/%m/%Y"))
}

fn iso_format_timestamp(timestamp: &NaiveDate) -> String {
    // iso date format by chrono
    // date + time
    // let formatted_timestamp = format!("{}", timestamp.format("%+"));

    // only date
    format!("{}", timestamp.format("%F"))
}

fn render_tasks_table(grouped_tasks: &Vec<(&String, &Vec<Task>)>) {
    let mut tasks_table = Table::new();

    tasks_table
        .load_preset(comfy_table::presets::ASCII_FULL)
        .set_content_arrangement(ContentArrangement::DynamicFullWidth)
        .set_width(100);

    let header_cell = |title: &str| {
        Cell::new(title)
            .fg(Color::Rgb {
                r: 205,
                g: 214,
                b: 244,
            })
            .bg(Color::Rgb {
                r: 49,
                g: 50,
                b: 68,
            })
            .add_attribute(Attribute::Bold)
    };

    tasks_table.set_header(vec![
        header_cell(" Date "),
        header_cell(" Description "),
        header_cell(" Status "),
        header_cell(" Id "),
    ]);

    let mut last_used_date = "";
    for (date, tasks) in grouped_tasks.iter() {
        for task in tasks.iter() {
            let display_date = if date.as_str() == last_used_date {
                ""
            } else {
                date
            };

            println!("{date} {display_date}");

            tasks_table.add_row(vec![
                Cell::new(display_date),
                Cell::new(&task.description).fg(Color::Red),
                Cell::new(&task.status),
                Cell::new(&task.id),
            ]);

            last_used_date = date;
        }
    }

    println!("{tasks_table}");
}

fn main() -> Result<(), Box<Error>> {
    let db_conn = open_db_connection().expect("Failed open storage connection");

    create_task_table(&db_conn).expect("Failed to create table");

    let cmd_matches = Command::new("Daily Dose")
        .version("1.0.0")
        .about("Record your daily dose of pain")
        .subcommands([
            Command::new("list")
                .about("List multiple standups based on timeline")
                .args([
                    arg!(-m --month <MONTH_NO> "List standups for specified month (eg. 1, 2, 3)")
                        .value_parser(value_parser!(u32).range(1..=12))
                        .required(false),
                    arg!(-l --limit <LIMIT> "Limit no. of standups in result")
                        .value_parser(value_parser!(u32).range(1..))
                        .required(false),
                    arg!(-s --search <QUERY> "Search query(task) for searching")
                        .value_parser(builder::NonEmptyStringValueParser::new())
                        .required(false),
                ]),
            Command::new("show")
                .about("Show tasks for any specific date")
                .args([
                    arg!(-d --day <DAY_NO> "Day for which fetching standup")
                        .value_parser(value_parser!(u32).range(1..=31))
                        .required(false),
                    arg!(-m --month <MONTH_NO> "Month for which fetching standup")
                        .value_parser(value_parser!(u32).range(1..=12))
                        .required(false),
                    arg!(-y --year <YEAR_NO> "Year for which fetching standup")
                        .value_parser(value_parser!(u32).range(1978..))
                        .required(false),
                ]),
            Command::new("add")
                .about("Add a task to current or specific date's standup task list")
                .args([
                    arg!([TASK] "Task description")
                        .value_parser(builder::NonEmptyStringValueParser::new())
                        .required(true),
                    arg!(-d --day <DAY_NO> "Day for which fetching standup")
                        .value_parser(value_parser!(u32).range(1..=31))
                        .required(false),
                    arg!(-m --month <MONTH_NO> "Month for which fetching standup")
                        .value_parser(value_parser!(u32).range(1..=12))
                        .required(false),
                    arg!(-y --year <YEAR_NO> "Year for which fetching standup")
                        .value_parser(value_parser!(u32).range(1978..))
                        .required(false),
                ]),
            Command::new("update")
                .about("Update a task based on task id")
                .args([
                    arg!([TASK] "Task description")
                        .value_parser(builder::NonEmptyStringValueParser::new())
                        .required(true),
                    arg!(--id <TASK_ID> "Task ID to update on")
                        .value_parser(builder::NonEmptyStringValueParser::new()),
                ]),
            Command::new("mark")
                .about("Mark today's specific task as done")
                .arg(
                    arg!([TASK_INDEX] "Mark current date's task based on task index")
                        .value_parser(value_parser!(u8).range(1..=100))
                        .required(true),
                ),
            Command::new("unmark")
                .about("Unmark today's specific task as todo")
                .arg(
                    arg!([TASK_INDEX] "Mark current date's task based on task index")
                        .value_parser(value_parser!(u8).range(1..=100))
                        .required(true),
                ),
            Command::new("delete")
                .about("Delete a task based on task id")
                .arg(
                    arg!(--id <TASK_ID> "Task ID to delete")
                        .value_parser(builder::NonEmptyStringValueParser::new()),
                ),
        ])
        .get_matches();

    if let Some(list_sub_cmd_matches) = cmd_matches.subcommand_matches("list") {
        let mut now = Local::now().date_naive();

        if let Some(month_no) = list_sub_cmd_matches.get_one::<u32>("month") {
            now = now.with_month(*month_no).expect("Invalid month");
        }

        let first_day_of_month = now.with_day(1).expect("Internal Error: Invalid day");

        // https://docs.rs/rusqlite/latest/rusqlite/struct.Statement.html#use-with-positional-parameters-1
        let mut stmt = db_conn.prepare(
            "SELECT id, description, status, date FROM tasks WHERE date BETWEEN :start_date AND :end_date ORDER BY date DESC",
        )?;

        let rows = stmt.query_map(
            [
                iso_format_timestamp(&first_day_of_month),
                iso_format_timestamp(&now),
            ],
            |row| {
                Ok(Task {
                    id: row.get(0)?,
                    description: row.get(1)?,
                    status: row.get(2)?,
                    date: row.get(3)?,
                })
            },
        )?;

        let mut date_tasks_map: HashMap<String, Vec<Task>> = HashMap::new();
        for r in rows {
            match r {
                Ok(task) => {
                    if date_tasks_map.contains_key(&task.date) {
                        let task_list = date_tasks_map.get_mut(&task.date);

                        match task_list {
                            Some(list) => {
                                list.push(task);
                            }
                            None => {
                                date_tasks_map.insert(task.date.clone(), vec![task]);
                            }
                        }
                    } else {
                        date_tasks_map.insert(task.date.clone(), vec![task]);
                    }
                }
                Err(err) => {
                    println!("Error accessing row = {:?}", err);
                    continue;
                }
            }
        }

        let mut task_grouped_by_date: Vec<(&String, &Vec<Task>)> = date_tasks_map.iter().collect();

        // sorting by date
        task_grouped_by_date.sort_by(|a, b| b.0.cmp(a.0));

        render_tasks_table(&task_grouped_by_date);
    }

    if let Some(show_sub_cmd_matches) = cmd_matches.subcommand_matches("show") {
        // list matches
        println!("Show sub command matches = {:?}", show_sub_cmd_matches);
    }

    if let Some(add_sub_cmd_matches) = cmd_matches.subcommand_matches("add") {
        let task_description = add_sub_cmd_matches
            .get_one::<String>("TASK")
            .expect("Task description is required for add");
        println!("Task description = {}", task_description);

        let timestamp = construct_timestamp(add_sub_cmd_matches);

        let iso_timestamp = iso_format_timestamp(&timestamp);

        let uid = Ulid::new();

        if let Err(error) = db_conn.execute(
            "insert into tasks (id, description, status, date) values (?1, ?2, ?3, ?4)",
            (
                &uid.to_string(),
                &task_description,
                Status::Todo,
                &iso_timestamp,
            ),
        ) {
            println!("Error inserting new task = {:?}", error);
        }
    }

    if let Some(update_sub_cmd_matches) = cmd_matches.subcommand_matches("update") {
        // list matches
        println!("Update sub command matches = {:?}", update_sub_cmd_matches);
    }

    if let Some(mark_sub_cmd_matches) = cmd_matches.subcommand_matches("mark") {
        // list matches
        println!("Mark sub command matches = {:?}", mark_sub_cmd_matches);
    }

    if let Some(unmark_sub_cmd_matches) = cmd_matches.subcommand_matches("unmark") {
        // list matches
        println!("Unmark sub command matches = {:?}", unmark_sub_cmd_matches);
    }

    if let Some(deleted_sub_cmd_matches) = cmd_matches.subcommand_matches("delete") {
        // list matches
        println!(
            "Deleted sub command matches = {:?}",
            deleted_sub_cmd_matches
        );
    }

    Ok(())
}
