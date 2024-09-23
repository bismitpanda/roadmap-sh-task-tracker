use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};
use ulid::Ulid;

enum Commands {
    Add,
    Update,
    Delete,
    Mark,
    List,
}

#[derive(Debug)]
pub enum CliError {
    InvalidCommand,
    InvalidArgs,
}

impl FromStr for Commands {
    type Err = CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "add" => Ok(Self::Add),
            "update" => Ok(Self::Update),
            "delete" => Ok(Self::Delete),
            "mark" => Ok(Self::Mark),
            "list" => Ok(Self::List),
            _ => Err(CliError::InvalidCommand),
        }
    }
}

#[derive(Deserialize, Serialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "kebab-case")]
enum Status {
    InProgress,
    Done,
    ToDo,
}

impl FromStr for Status {
    type Err = CliError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "in-progress" => Ok(Self::InProgress),
            "done" => Ok(Self::Done),
            "todo" => Ok(Self::ToDo),
            _ => Err(CliError::InvalidArgs),
        }
    }
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Done => write!(f, "done"),
            Self::ToDo => write!(f, "todo"),
            Self::InProgress => write!(f, "in-progress"),
        }
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Task {
    id: Ulid,
    description: String,
    status: Status,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

fn print_help() {
    const HELP_TEXT: &str = r#"Usage: task-cli [command] [args]

Commands:
    add      Adds a new task
    update   Update a task
    delete   Delete a task
    mark     Change status of a task
    list     List all tasks"#;

    println!("{HELP_TEXT}");
}

fn main() {
    let args = std::env::args().collect::<Vec<_>>();

    if args.len() == 1 {
        print_help();
    } else {
        if let Ok(cmd) = Commands::from_str(&args[1]) {
            let mut tasks =
                if let Ok(tasks) = std::fs::read(dirs::home_dir().unwrap().join(".tasks.json")) {
                    serde_json::from_slice::<Vec<Task>>(&tasks).expect("invalid json format")
                } else {
                    Vec::new()
                };

            match cmd {
                Commands::Add => {
                    let description = args[2].clone();

                    let new_task = Task {
                        id: Ulid::new(),
                        description,
                        status: Status::ToDo,
                        created_at: Utc::now(),
                        updated_at: Utc::now(),
                    };

                    tasks.push(new_task);
                }

                Commands::List => {
                    let tasks = if let Some(status) = args.get(2) {
                        let status = Status::from_str(status).expect("invalid status type");
                        tasks
                            .iter()
                            .filter(|task| task.status == status)
                            .cloned()
                            .collect()
                    } else {
                        tasks.clone()
                    };

                    for task in tasks {
                        println!("{}. {} ({})", task.id, task.description, task.status)
                    }
                }

                Commands::Mark => {
                    let id = Ulid::from_string(&args[2]).expect("invalid ulid format");
                    let status = Status::from_str(&args[3]).expect("invalid status kind");

                    for task in tasks.iter_mut() {
                        if task.id == id {
                            task.status = status;
                            break;
                        }
                    }
                }

                Commands::Delete => {
                    let id = Ulid::from_string(&args[2]).expect("invalid ulid format");
                    tasks.retain(|task| task.id != id);
                }

                Commands::Update => {
                    let id = Ulid::from_string(&args[2]).expect("invalid ulid format");
                    let new_description = args[3].clone();

                    for task in tasks.iter_mut() {
                        if task.id == id {
                            task.description = new_description;
                            break;
                        }
                    }
                }
            }

            std::fs::write(
                dirs::home_dir().unwrap().join(".tasks.json"),
                serde_json::to_vec(&tasks).expect("could not convert to json"),
            )
            .expect("could not write to tasks file");
        } else {
            println!("Invalid command");
            print_help();
        }
    }
}
