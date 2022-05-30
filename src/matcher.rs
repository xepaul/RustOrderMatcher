pub mod matcheri {
  use im::hashmap::HashMap;
  use itertools::Itertools;
  use std::fmt;

  #[derive(Debug, PartialEq, Eq, Copy, Clone)]
  pub enum OrderType {
    Buy,
    Sell,
  }
  #[derive(Debug, Copy, Clone, Eq, PartialEq)]
  pub struct Order {
    pub id: usize,
    pub(crate) order_type: OrderType,
    pub price: u32,
    pub quantity: u32,
  }
  #[derive(Debug, Eq, PartialEq)]
  pub struct Trade {
    pub(crate) buy_id: usize,
    pub sell_id: usize,
    pub price: u32, // this should be the sell price.
    pub quantity_traded: u32,
  }
  impl fmt::Display for Trade {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      write!(
        f,
        "Trade: {0:?} BTC @ {1:?} USD between {2:?} and {3:?}",
        self.quantity_traded, self.price, self.buy_id, self.sell_id
      )
    }
  }

  pub fn match_trades(orders: &Vec<Order>) -> (Vec<Order>, Vec<Trade>) {
    let order_map: HashMap<usize, Order> = orders
      .into_iter()
      .enumerate()
      .map(|(a, b)| (a, *b))
      .collect();
    return go(&order_map);
    fn go(orders: &HashMap<usize, Order>) -> (Vec<Order>, Vec<Trade>) {
      (|| {
        let v = orders
          .iter()
          .filter(|(_, b)| b.order_type == OrderType::Sell)
          .sorted_by(|a, b| Ord::cmp(&b.0, &a.0))
          .min_by(|x, y| x.1.price.cmp(&y.1.price))?;
        let b = orders
          .iter()
          .filter(|(_, b)| b.order_type == OrderType::Buy)
          .sorted_by(|a, b| Ord::cmp(&b.0, &a.0))
          .rev()
          .find(|x| v.1.price <= x.1.price)?;
        let trade_quantity = b.1.quantity.min(v.1.quantity);
        let update_orders = |orders: &HashMap<usize, Order>, id: usize| {
          let update_order = |o: Order| -> Option<Order> {
            let n_quantity = o.quantity - trade_quantity;
            if n_quantity > 0 {
              Some(Order {
                quantity: n_quantity,
                ..o
              })
            } else {
              None
            }
          };
          return orders.alter(|oo| oo.and_then(update_order), id);
        };
        let n1_orders = update_orders(orders, *b.0);
        let n_orders = update_orders(&n1_orders, *v.0);

        let new_trade = Trade {
          buy_id: b.1.id,
          sell_id: v.1.id,
          price: v.1.price,
          quantity_traded: trade_quantity,
        };
        let (partials, mut building) = go(&n_orders);
        building.insert(0, new_trade);
        return Some((partials, building));
      })()
      .unwrap_or_else(|| {
        let p: Vec<Order> = orders
          .iter()
          .sorted_by(|a, b| Ord::cmp(&b.0, &a.0))
          .rev()
          .map(|x| *x.1)
          .collect();
        (p, Vec::new())
      })
    }
  }

  pub fn match_trades2(orders: &Vec<Order>) -> (Vec<Order>, Vec<Trade>) {
    return go(&orders);
    fn go(orders: &Vec<Order>) -> (Vec<Order>, Vec<Trade>) {
      (|| {
        let v = orders
          .iter()
          .filter(|b| b.order_type == OrderType::Sell)
          .min_by(|x, y| x.price.cmp(&y.price))?;
        let b = orders
          .iter()
          .filter(|b| b.order_type == OrderType::Buy)
          .find(|x| v.price <= x.price)?;
        let trade_quantity = b.quantity.min(v.quantity);

        let update_order = |o: Order| -> Option<Order> {
          let n_quantity = o.quantity - trade_quantity;
          if n_quantity > 0 {
            Some(Order {
              quantity: n_quantity,
              ..o
            })
          } else {
            None
          }
        };
        let new_orders: Vec<Order> = orders // should use mut_retain
          .into_iter()
          .filter_map(|o| {
            if (*o).id == v.id || (*o).id == b.id {
              update_order(*o)
            } else {
              Some(*o)
            }
          })
          .collect();
        let new_trade = Trade {
          buy_id: b.id,
          sell_id: v.id,
          price: v.price,
          quantity_traded: trade_quantity,
        };
        let (partials, mut building) = go(&new_orders);
        building.insert(0, new_trade);
        return Some((partials, building));
      })()
      .unwrap_or_else(|| (orders.to_vec(), Vec::new()))
    }
  }

  use futures::stream::{self, StreamExt};
  use futures::{future, Stream};
  pub fn process_orders(stream: impl Stream<Item = Order>) -> impl Stream<Item = Trade> {
    stream
      .scan(Vec::new(), |s: &mut std::vec::Vec<Order>, y| {
        s.push(y);
        if y.order_type == OrderType::Buy {
          let (o, trades) = match_trades(&s);
          // for key in o.iter() {
          //   println!("Partials: {key:?} ");
          // }
          *s = o;
          future::ready(Some(Some(trades)))
        } else {
          future::ready(Some(None))
        }
      })
      .filter_map(|x| future::ready(x))     
      .flat_map(|x| stream::iter(x))
  }
}

#[cfg(test)]
pub mod matcher_processor_tests {

  use super::matcheri::{
    match_trades, match_trades2, process_orders, Order,
    OrderType::{self, Buy, Sell},
    Trade,
  };
  pub use crate::orderparser::parse_order;

  use futures::stream::{self, StreamExt};
  #[actix_rt::test]
  async fn test_simple_processor() {
    let orders1 = [
      Order {
        id: 0,
        order_type: OrderType::Sell,
        price: 5000,
        quantity: 100,
      },
      Order {
        id: 1,
        order_type: OrderType::Buy,
        price: 6000,
        quantity: 50,
      },
    ];
    let stream = process_orders(stream::iter(orders1));
    let t1: Trade = Trade {
      buy_id: 1,
      sell_id: 0,
      price: 5000,
      quantity_traded: 50,
    };

    assert_eq!(vec![t1], stream.collect::<Vec<_>>().await);
  }

  #[actix_rt::test]
  async fn test_simple_processor2() {
    let orders1 = [
      Order {
        id: 1,
        order_type: OrderType::Sell,
        price: 5001,
        quantity: 100,
      },
      Order {
        id: 2,
        order_type: OrderType::Sell,
        price: 5000,
        quantity: 25,
      },
      Order {
        id: 3,
        order_type: OrderType::Buy,
        price: 6000,
        quantity: 50,
      },
    ];
    let stream = process_orders(stream::iter(orders1));
    let trades: Vec<Trade> = [
      Trade {
        buy_id: 3,
        sell_id: 2,
        price: 5000,
        quantity_traded: 25,
      },
      Trade {
        buy_id: 3,
        sell_id: 1,
        price: 5001,
        quantity_traded: 25,
      },
    ]
    .into_iter()
    .collect();

    assert_eq!(trades, stream.collect::<Vec<_>>().await);
  }

  #[actix_rt::test]
  async fn test_simple_processor3() {
    let orders1 = [
      Order {
        id: 1,
        order_type: OrderType::Sell,
        price: 5000,
        quantity: 75,
      },
      Order {
        id: 2,
        order_type: OrderType::Buy,
        price: 6000,
        quantity: 50,
      },
      Order {
        id: 3,
        order_type: OrderType::Buy,
        price: 6000,
        quantity: 50,
      },
    ];
    let stream = process_orders(stream::iter(orders1));
    let t1: Vec<Trade> = [
      Trade {
        buy_id: 2,
        sell_id: 1,
        price: 5000,
        quantity_traded: 50,
      },
      Trade {
        buy_id: 3,
        sell_id: 1,
        price: 5000,
        quantity_traded: 25,
      },
    ]
    .into_iter()
    .collect();

    assert_eq!(t1, stream.collect::<Vec<_>>().await);
  }

}
pub mod matcher_tests {
  use super::matcheri::{
    match_trades, match_trades2, process_orders, Order,
    OrderType::{self, Buy, Sell},
    Trade,
  };
  pub use crate::orderparser::parse_order;

  #[test]
  fn test_simple() {
    let orders1 = [
      Order {
        id: 0,
        order_type: OrderType::Sell,
        price: 5000,
        quantity: 100,
      },
      Order {
        id: 1,
        order_type: OrderType::Buy,
        price: 6000,
        quantity: 50,
      },
    ];
    let (o, r) = match_trades(&orders1.into_iter().collect());
    let expected: Vec<Trade> = [Trade {
      buy_id: 1,
      sell_id: 0,
      price: 5000,
      quantity_traded: 50,
    }]
    .into_iter()
    .collect();
    let expected_order: Vec<Order> = [Order {
      id: 0,
      order_type: Sell,
      price: 5000,
      quantity: 50,
    }]
    .into_iter()
    .collect();
    assert_eq!(expected, r, "testing trade output");
    assert_eq!(expected_order, o, "testing trade output")
  }

  #[test]
  fn test_simple2() {
    let orders1 = [
      Order {
        id: 1,
        order_type: OrderType::Sell,
        price: 5001,
        quantity: 100,
      },
      Order {
        id: 2,
        order_type: OrderType::Sell,
        price: 5000,
        quantity: 25,
      },
      Order {
        id: 3,
        order_type: OrderType::Buy,
        price: 6000,
        quantity: 50,
      },
    ];
    let (o, r) = match_trades(&orders1.into_iter().collect());
    let expected: Vec<Trade> = [
      Trade {
        buy_id: 3,
        sell_id: 2,
        price: 5000,
        quantity_traded: 25,
      },
      Trade {
        buy_id: 3,
        sell_id: 1,
        price: 5001,
        quantity_traded: 25,
      },
    ]
    .into_iter()
    .collect();
    let expected_order: Vec<Order> = [Order {
      id: 1,
      order_type: Sell,
      price: 5001,
      quantity: 75,
    }]
    .into_iter()
    .collect();
    assert_eq!(r, expected);
    assert_eq!(o, expected_order)
  }

  #[test]
  fn test_simple3() {
    let orders1 = [
      Order {
        id: 1,
        order_type: OrderType::Sell,
        price: 5000,
        quantity: 75,
      },
      Order {
        id: 2,
        order_type: OrderType::Buy,
        price: 6000,
        quantity: 50,
      },
      Order {
        id: 3,
        order_type: OrderType::Buy,
        price: 6000,
        quantity: 50,
      },
    ];
    let (o, r) = match_trades(&orders1.into_iter().collect());
    let expected: Vec<Trade> = [
      Trade {
        buy_id: 2,
        sell_id: 1,
        price: 5000,
        quantity_traded: 50,
      },
      Trade {
        buy_id: 3,
        sell_id: 1,
        price: 5000,
        quantity_traded: 25,
      },
    ]
    .into_iter()
    .collect();
    let expected_order: Vec<Order> = [Order {
      id: 3,
      order_type: Buy,
      price: 6000,
      quantity: 25,
    }]
    .into_iter()
    .collect();
    assert_eq!(r, expected);
    assert_eq!(o, expected_order)
  }
}
