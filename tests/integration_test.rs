use sqlx::sqlite::SqlitePoolOptions;

#[tokio::test]
async fn test_database_migration() {
    // Create an in-memory database for testing
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("Failed to connect to database");

    // Run migrations
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Test inserting a user
    sqlx::query("INSERT INTO users (id, points, username) VALUES (?, ?, ?)")
        .bind(12345i64)
        .bind(10.0f64)
        .bind("testuser")
        .execute(&pool)
        .await
        .expect("Failed to insert user");

    // Test inserting a stock
    sqlx::query("INSERT INTO user_stocks (user_id, stock_symbol, shares, avg_price) VALUES (?, ?, ?, ?)")
        .bind(12345i64)
        .bind("AAPL")
        .bind(10i64)
        .bind(1.50f64)
        .execute(&pool)
        .await
        .expect("Failed to insert stock");

    // Test querying the stock
    let result: (i64, String, i64, f64) = sqlx::query_as("SELECT user_id, stock_symbol, shares, avg_price FROM user_stocks WHERE user_id = ?")
        .bind(12345i64)
        .fetch_one(&pool)
        .await
        .expect("Failed to query stock");

    assert_eq!(result.0, 12345);
    assert_eq!(result.1, "AAPL");
    assert_eq!(result.2, 10);
    assert!((result.3 - 1.50).abs() < 0.001);

    println!("Database migration test passed!");
}