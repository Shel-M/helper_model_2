use std::{fs, path::Path};

use tokio::sync::OnceCell;
use tokio::test;
use tracing::{debug, info, Level};

use crate::{user::User, DB};

static INIT: OnceCell<DB> = OnceCell::const_new();

async fn init_test() -> DB {
    INIT.get_or_init(|| async {
        init_log();
        let db = init_db().await;

        db
    })
    .await
    .clone()
}

fn init_log() {
    tracing_subscriber::fmt()
        .with_max_level(Level::TRACE)
        .with_line_number(true)
        .with_file(true)
        .with_test_writer()
        .init();

    info!("Starting log...");
}

async fn init_db() -> DB {
    let test_db = Path::new("test.db");
    if fs::exists(test_db).is_ok_and(|e| e) {
        fs::remove_file(test_db).expect("Could not delete existant test.db");
    }

    fs::File::create(test_db).expect("Could not create new test.db");
    let db = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect(
            test_db
                .to_str()
                .expect("Could not convert test.db path to string (???)"),
        );

    let db = db.await.expect("Could not initialize database");

    sqlx::migrate!("./migrations")
        .run(&db)
        .await
        .expect("Failed to run migrations");

    db
}

#[test]
async fn test_init() {
    let db = init_test().await;

    let u = User::new("test", None);
    let u = u.insert(&db).await.expect("Couldn't insert user");

    let user = User::get_by_name(&db, "test")
        .await
        .unwrap_or_else(|_| panic!("Couldn't get user {u:?} from db"));
    let user = user.first().expect("No user found");
    debug!("User: {user:?}");

    assert_eq!(u.id, user.id);
}
