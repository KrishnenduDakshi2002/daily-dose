use std::fs;

use chrono::{DateTime, Datelike, Utc};
use clap::{arg, builder, command, error::ErrorKind, value_parser, ArgAction, ArgMatches, Command};
use rusqlite::{types::ToSqlOutput, Connection, Error, ToSql};
use strum::Display;
use ulid::Ulid;

#[derive(Display)]
#[strum(serialize_all = "snake_case")]
enum Status {
    Todo,
    Completed,
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

fn construct_timestamp(arg_matches: &ArgMatches) -> DateTime<Utc> {
    let mut timestamp = Utc::now();
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

fn display_format_timestamp(timestamp: &DateTime<Utc>) -> String {
    format!("{}", timestamp.format("%d/%m/%Y"))
}

fn iso_format_timestamp(timestamp: &DateTime<Utc>) -> String {
    // iso date format by chrono
    // date + time
    // let formatted_timestamp = format!("{}", timestamp.format("%+"));

    // only date
    format!("{}", timestamp.format("%F"))
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
                    arg!(--week <WEEK_NO> "List standups for specified week (eg. 1, 2, 3)")
                        .value_parser(value_parser!(u32).range(1..=4))
                        .required(false),
                    arg!(--month <MONTH_NO> "List standups for specified month (eg. 1, 2, 3)")
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
        if let Some(week) = list_sub_cmd_matches.get_one::<u8>("week") {
            println!("week = {}", week);
        }

        if let Some(month) = list_sub_cmd_matches.get_one::<u8>("month") {
            println!("month = {}", month);
        }
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

        let display_formatted_timestamp = display_format_timestamp(&timestamp);
        println!("Timestamp = {}", &display_formatted_timestamp);

        let iso_timestamp = iso_format_timestamp(&timestamp);
        println!("ISO Timestamp = {}", &iso_timestamp);

        let uid = Ulid::new();

        db_conn.execute(
            "insert into tasks (id, description, status, date) values (?1, ?2, ?3, ?4)",
            (
                &uid.to_string(),
                &task_description,
                Status::Todo,
                &iso_timestamp,
            ),
        )?;
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
