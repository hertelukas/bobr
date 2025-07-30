-- Add migration script here
CREATE TABLE lmsr_market (
    id INTEGER PRIMARY KEY,
    liquidity REAL NOT NULL,
    is_resolved BOOLEAN NOT NULL DEFAULT FALSE,
    resolved_idx INTEGER,
    market_volume REAL NOT NULL DEFAULT 0,
    title TEXT NOT NULL UNIQUE,
    description TEXT
);

CREATE TABLE shares (
    market_id INTEGER NOT NULL,
    idx INTEGER NOT NULL,
    amount INTEGER NOT NULL DEFAULT 0,
    description TEXT NOT NULL,
    PRIMARY KEY(market_id, idx),
    FOREIGN KEY(market_id) REFERENCES lmsr_market(id) ON DELETE CASCADE
);

CREATE TABLE users (
    id INTEGER PRIMARY KEY,
    points REAL NOT NULL,
    username TEXT NOT NULL UNIQUE
);

CREATE TABLE user_owns (
    user_id INTEGER NOT NULL,
    market_id INTEGER NOT NULL,
    share_idx INTEGER NOT NULL,
    amount INTEGER NOT NULL,
    PRIMARY KEY(user_id, market_id, share_idx),
    FOREIGN KEY(user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY(market_id) REFERENCES lmsr_market(id) ON DELETE CASCADE
);
