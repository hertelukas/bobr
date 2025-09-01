# Stock Trading Feature

This document describes the new stock trading feature added to the bobr Discord bot.

## Setup

1. Add the following environment variable to your `.env` file:
   ```
   FINNHUB_API_KEY=your_finnhub_api_key_here
   ```

2. Get a free Finnhub API key from https://finnhub.io/register

3. The stock trading tables will be created automatically when you run the bot.

## Commands

### `/buy_stock <symbol> <shares>`
Buy virtual stocks using your points.

- `symbol`: Stock symbol (e.g., AAPL, TSLA, MSFT)
- `shares`: Number of shares to buy

Example: `/buy_stock AAPL 10`

**Note**: Stock prices are converted from USD to points (1 USD = 100 points).

### `/stock_portfolio [user]`
View your stock portfolio or another user's portfolio.

- `user`: Optional Discord user to view portfolio of (defaults to yourself)

Shows:
- Current stock holdings
- Current stock prices
- Profit/loss for each holding
- Total portfolio value

### `/stock_price <symbol>`
Get current stock price and information.

- `symbol`: Stock symbol (e.g., AAPL, TSLA, MSFT)

Shows:
- Current price
- Previous close
- Day's change (amount and percentage)
- Day high/low
- Opening price

## How It Works

1. **Points System**: Uses the existing bot point system (users get 0.01 points per message, start with 10 points)

2. **Price Conversion**: Stock prices in USD are divided by 100 to convert to points (e.g., a $150 stock costs 1.5 points)

3. **Real-time Data**: Uses Finnhub API to fetch real-time stock prices

4. **Portfolio Tracking**: Tracks average cost basis and calculates profit/loss

5. **Database**: Stores stock ownership in a new `user_stocks` table

## Examples

```
User has 5.0 points (500 "cents")

/stock_price AAPL
> AAPL Stock Price
> Current Price: $150.00
> Previous Close: $148.50
> Change: 📈+1.50 (+1.01%)

/buy_stock AAPL 1
> Successfully bought 1 shares of AAPL at $150.00 per share for a total of $150.00

/stock_portfolio
> Shows portfolio with AAPL holding and current value
```

## Error Handling

- Invalid stock symbols are rejected
- Insufficient points prevent purchases
- Network errors are handled gracefully
- Real-time price updates in portfolio view