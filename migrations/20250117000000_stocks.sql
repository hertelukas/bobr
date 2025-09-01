-- Add migration script here for stock trading functionality
CREATE TABLE user_stocks (
    user_id INTEGER NOT NULL,
    stock_symbol TEXT NOT NULL,
    shares INTEGER NOT NULL DEFAULT 0,
    avg_price REAL NOT NULL,
    PRIMARY KEY(user_id, stock_symbol),
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE
);