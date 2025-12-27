use clap::{arg, builder, command, error::ErrorKind, value_parser, ArgAction, Command};

fn main() {
    let cmd_matches = Command::new("Daily Dose")
        .version("1.0.0")
        .about("Record your daily dose of pain")
        .subcommands([
            Command::new("list")
                .about("List multiple standups based on timeline")
                .args([
                    arg!(--week <WEEK_NO> "List standups for specified week (eg. 1, 2, 3)")
                        .value_parser(value_parser!(u8).range(1..=4))
                        .required(false),
                    arg!(--month <MONTH_NO> "List standups for specified month (eg. 1, 2, 3)")
                        .value_parser(value_parser!(u8).range(1..=12))
                        .required(false),
                    arg!(-l --limit <LIMIT> "Limit no. of standups in result")
                        .value_parser(value_parser!(u16).range(1..))
                        .required(false),
                    arg!(-s --search <QUERY> "Search query(task) for searching")
                        .value_parser(builder::NonEmptyStringValueParser::new())
                        .required(false),
                ]),
            Command::new("show")
                .about("Show tasks for any specific date")
                .args([
                    arg!(-d --day <DAY_NO> "Day for which fetching standup")
                        .value_parser(value_parser!(u8).range(1..=31))
                        .required(false),
                    arg!(-m --month <MONTH_NO> "Month for which fetching standup")
                        .value_parser(value_parser!(u8).range(1..=12))
                        .required(false),
                    arg!(-y --year <YEAR_NO> "Year for which fetching standup")
                        .value_parser(value_parser!(u16).range(1978..))
                        .required(false),
                ]),
            Command::new("add")
                .about("Add a task to current or specific date's standup task list")
                .args([
                    arg!([TASK] "Task description")
                        .value_parser(builder::NonEmptyStringValueParser::new())
                        .required(true),
                    arg!(-d --day <DAY_NO> "Day for which fetching standup")
                        .value_parser(value_parser!(u8).range(1..=31))
                        .required(false),
                    arg!(-m --month <MONTH_NO> "Month for which fetching standup")
                        .value_parser(value_parser!(u8).range(1..=12))
                        .required(false),
                    arg!(-y --year <YEAR_NO> "Year for which fetching standup")
                        .value_parser(value_parser!(u16).range(1978..))
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
}
