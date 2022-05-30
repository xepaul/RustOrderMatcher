#Matching Orders

Suppose you run a trading site for bitcoin to match orders to buy against orders to sell. We want to emit trade messages every time a buy order trades with a sell order.
When an order to sell comes in we need to store them until an order to buy comes in.
When an order to buy comes in, it should match against one (or more) orders to sell. The matching algorithm is as follows:
 1. Start with the sell order with the lowest price.
2. Compare it to the price of the buy order.
3. If that sell price is less than or equal to the buy price, we emit a trade
message and reduce the quantities on both orders.
4. If the sell order reaches zero quantity, it is removed.
5. Keep doing this until there are no more orders matching or the buy order
has been completely fulfilled.
Each order consists of the following information:
  order type: buy or sell.
  price: [1, 99999],
  quantity: [1, 999],
To get you started, here are some of the structures you'll need. Feel free to add attributes, make changes as required.
```
 enum OrderType {
     Buy,
     Sell, }
 struct Order {
     id: usize,
     order_type: OrderType,
     price: u32,
     quantity: u32
}
 struct Trade {
     buy_id: usize,
     sell_id: usize,
     price: u32, // this should be the sell price.
     quantity_traded: u32
}
```
# Inputs and Outputs

You should take orders from stdin. Each line of input will be in the form {id}: {Buy|Sell} {quantity} BTC @ {price} .
   
 You should output to stdout an array of trades of the form
 Trade: {quantity} BTC @ {price} between {buy_id} and {sell_id}
Examples
Here are some example orders that illustrate how it should work. Note that these are not exhaustive of every possible case.
1. Simple trade: Input:
  1: Sell 100 BTC @ 5000 USD
  2: Buy 50 BTC @ 6000 USD
Output:
  Trade: 50 BTC @ 5000 USD between 2 and 1
Notice that the price is the lower of the two prices (the sell price), 2. Multiple Trades for a single order
Input:
  1: Sell 100 BTC @ 5001 USD
  2: Sell 25 BTC @ 5000 USD
  3: Buy 50 BTC @ 6000 USD
Output:
  Trade 25 BTC @ 5000 USD between 3 and 2
  Trade 25 BTC @ 5001 between 3 and 1
We have to match 2 orders to fill the entire order. Notice it matches the order with lowest sell price first.
3. Not enough orders

Input:
```     
  1: Sell 75 BTC @ 5000 USD
  2: Buy 50 BTC @ 6000 USD
  3: Buy 50 BTC @ 6000 USD
  ```     
Output:
```     
  Trade 50 BTC @ 5000 USD between 2 and 1
  Trade 25 BTC @ 5000 USD between 3 and 1
```
Note that because we used up some of the sell order with the first buy order, we don't have enough to fill up the second.


# Running console app

Run with 
```cargo run```

Quit by entering a blank line
