use tracing::info;

use crate::{
    context::Context,
    utils::user_utils::{get_inactive_users, get_restricted_users, remove_ranking},
};

pub async fn run_cleanup(ctx: Context) {
    let restricted_users = get_restricted_users(&ctx.pool).await;

    if let Err(why) = restricted_users {
        println!("Error: {:?}", why);
        return;
    }

    let users = restricted_users.unwrap();

    info!("found {} restricted users", users.len());
    for user in users {
        remove_ranking(&ctx.redis, &user).await;
    }

    let users = get_inactive_users(&ctx.pool).await;
    if let Err(why) = users {
        println!("Error: {:?}", why);
        return;
    }

    let users = users.unwrap();

    for user in users {
        remove_ranking(&ctx.redis, &user).await;
        info!("flagged user {} as active", user.username);
    }
}
