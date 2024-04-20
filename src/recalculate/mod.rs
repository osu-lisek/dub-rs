use std::sync::Arc;

use tokio::io::{self, AsyncBufReadExt, BufReader};
use tracing::error;
use tracing::info;
use tracing::warn;

use crate::context::Context;
use crate::db::user::User;
use crate::utils::beatmap_utils::get_beatmap_by_term;
use crate::utils::beatmap_utils::Beatmap;
use crate::utils::score_utils::get_score_with_beatmap_by_id;
use crate::utils::score_utils::UserScoreWithBeatmap;
use crate::utils::user_utils::find_user_by_id_or_username;

use self::processing::process_command;
mod processing;

pub struct CalculationQueue {
    pub users: Vec<User>,
    pub score: Vec<UserScoreWithBeatmap>,
    pub beatmaps: Vec<Beatmap>,
}

pub async fn help_command(
    command: String,
    arguments: Vec<String>,
    ctx: &Context,
    queue: &CalculationQueue,
) {
    println!(
        r#"
ADD <USER/SCORE/BEATMAP> <ID> # Adds provided argument to calculation queue
# User can be username/safe username/id
# Beatmap can be checksum or ID
# Score can be only ID

PREVIEW # Shows you what input has been added

REM <USER/SCORE/BEATMAP> <ID> # Sames as ADD, but removes it from calculaton result

PROCESS # Starts recalculation and proceses all inputs
"#
    );
}

pub async fn add_command(
    command: String,
    arguments: Vec<String>,
    ctx: &Context,
    queue: &mut CalculationQueue,
) {
    let add_type = arguments.get(0);

    if let None = add_type {
        error!("ADD: Type hasn't be provided, use HELP for details about this command");
        return;
    }

    let add_type = add_type.unwrap().to_owned();

    match add_type.as_str() {
        "USER" => {
            let term = arguments.get(1);

            if let None = term {
                error!("ADD: No term has been provided, consider providing username or user id");
                return;
            }

            let term = term.unwrap().to_owned();
            let user = find_user_by_id_or_username(&ctx.pool, term).await;

            if let Err(error) = user {
                error!("ADD: Error while fetching user: {:#?}", error);
                return;
            }

            let user = user.unwrap();

            if let None = user {
                error!("ADD: No user has been found by this term");
            }

            let user = user.unwrap();

            if queue.users.iter().find(|&x| x.id == user.id).is_some() {
                warn!("ADD: This user is already in calculation queue.");
                return;
            }
            //Adding it to queue
            queue.users.push(user.clone());

            info!(
                "ADD: Added user {} ({}) to calculation queue",
                user.username, user.id
            );
        }
        "SCORE" => {
            let term = arguments.get(1);

            if let None = term {
                error!("ADD: No term has been provided, consider providing username or user id");
                return;
            }

            let term = term.unwrap().to_owned().parse::<i32>().unwrap_or(0);

            let score = get_score_with_beatmap_by_id(&ctx.pool, term).await;

            if let Err(error) = score {
                error!("Error while fetching score: {:#?}", error);
                return;
            }

            let score = score.unwrap();

            if let None = score {
                error!("Seems like no score found with this ID");
                return;
            }

            let score = score.unwrap();

            queue.score.push(score.clone());
            info!(
                "added score set by {} on beatmap {} - {} with status {} ({})",
                score.user.username,
                score.beatmap.artist,
                score.beatmap.title,
                score.score.status,
                score.score.id
            );
        }
        "BEATMAP" => {
            let term = arguments.get(1);

            if let None = term {
                error!("ADD: No term has been provided, consider providing username or user id");
                return;
            }

            let term = term.unwrap().to_owned();

            let beatmap = get_beatmap_by_term(&ctx.pool, term).await;

            if let Err(error) = beatmap {
                error!("Error while fetching beatmap: {:#?}", error);
                return;
            }

            let beatmap = beatmap.unwrap();

            if let None = beatmap {
                error!("Seems like no beatmap found with this ID");
                return;
            }

            let beatmap = beatmap.unwrap();

            queue.beatmaps.push(beatmap.clone());
            info!("added beatmap {} - {}", beatmap.artist, beatmap.title);
        }
        _ => error!("ADD: Unknown add type"),
    }
}

pub async fn preview_command(
    command: String,
    arguments: Vec<String>,
    ctx: &Context,
    queue: &mut CalculationQueue,
) {
    let mut result = String::new();

    result += "Users: \n";
    if !queue.users.is_empty() {
        for user in queue.users.clone() {
            result += format!("  {} ({})\n", user.username, user.id).as_str();
        }
    } else {
        result += "  None"
    }

    result += "\nScores: \n";
    if !queue.score.is_empty() {
        for score in queue.score.clone() {
            result += format!(
                "  Score set by {} on beatmap {} - {} with status {} ({})\n",
                score.user.username,
                score.beatmap.artist,
                score.beatmap.title,
                score.score.status,
                score.score.id
            )
            .as_str();
        }
    } else {
        result += "  None"
    }

    result += "\nBeatmaps: \n";
    if !queue.beatmaps.is_empty() {
        for beatmap in queue.beatmaps.clone() {
            result += format!(
                "  Beatmap {} - {} with status {} ({})\n",
                beatmap.artist, beatmap.title, beatmap.status, beatmap.beatmap_id
            )
            .as_str();
        }
    } else {
        result += "  None"
    }

    println!("{}", result);
}

pub async fn handle_command(
    command: String,
    arguments: Vec<String>,
    ctx: &Context,
    queue: &mut CalculationQueue,
) {
    match command.as_str() {
        "HELP" => help_command(command, arguments, ctx, queue).await,
        "ADD" => add_command(command, arguments, ctx, queue).await,
        "PREVIEW" => preview_command(command, arguments, ctx, queue).await,
        "PROCESS" => process_command(command, arguments, ctx, queue).await,

        _ => error!("Unknown command"),
    }
}

pub async fn recalculate_terminal(ctx: Context) {
    let ctx = Arc::new(ctx);
    let mut buffer = String::new();
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin);

    let mut queue = CalculationQueue {
        beatmaps: Vec::new(),
        score: Vec::new(),
        users: Vec::new(),
    };

    info!(r#"You've entered the recalculation terminal, to get extra info run HELP command"#);

    loop {
        let line = reader.read_line(&mut buffer).await;

        if let Err(error) = line {
            error!(
                "Error encounted while reading stdin, terminal will be closed, error: {}",
                error
            );
            break;
        }

        let arguments: Vec<String> = buffer.split(" ").map(String::from).collect();
        let command_name = arguments.get(0).unwrap_or(&String::from("")).to_owned();
        let arguments: Vec<String> = arguments
            .iter()
            .skip(1)
            .map(|f| f.trim().to_owned())
            .collect();

        handle_command(command_name.trim().to_string(), arguments, &ctx, &mut queue).await;
        buffer = String::new();
    }
}
