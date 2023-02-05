use recorder::Logger;

#[tokio::main]
async fn main() {
    let db = std::env::var("DATABASE_URL").expect("DATABASE_URL enviroment variable missing");
    let password_path =
        std::env::var("PASSWORD_FILE").expect("PASSWORD_FILE enviroment variable missing");
    dbg!(&db);
    let password = std::fs::read_to_string(password_path).expect("Cannot read password file");
    let mut logger = Logger::from(&db, password.trim())
        .await
        .expect("Database connection could not be established");
    logger.record().await;
}
