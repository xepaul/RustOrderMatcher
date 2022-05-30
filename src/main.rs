
use futures::future;
use futures::stream::{self, StreamExt};
mod matcher;
use crate::matcher::matcheri::{process_orders};

mod orderparser;
use crate::orderparser::{parse_order_input};
use std::error::Error;
use std::io::{self, BufRead, Write};

#[async_std::main]
async fn main() -> Result<(), Box<dyn Error>> {
  let console_input = stream::unfold((), |_| async move {
    let stdin = io::stdin();
    io::stdout().flush().ok()?;
    let mut input = String::new();
    stdin.lock().read_line(&mut input).ok()?;
    if input.trim_end().is_empty() {
      None
    } else {
      Some((input.clone(), ()))
    }
  });  

  println!("Enter orders in the form Trade: {{quantity}} BTC @ {{price}} between {{buy_id}} and {{sell_id}}");

  process_orders(console_input.filter_map(|x| future::ready(parse_order_input(&x))))
    .for_each(|trade| {
      println!("{}", trade);
      future::ready(())
    })
    .await;
  return Ok(());  
}

//1: Sell 100 BTC @ 5000 USD
//2: Buy 50 BTC @ 6000 USD