
  use nom::branch::alt;
  use nom::bytes::complete::tag;
  use nom::character::complete::{digit1, newline, space1};
  use nom::combinator::{map, map_res};
  use nom::sequence::{preceded, terminated, tuple};
  use nom::IResult;

  use crate::matcher::matcheri::{Order, OrderType};

  pub fn parse_order(input: &str) -> IResult<&str, Order> {
    let parser_usize = map_res(digit1, |s| usize::from_str_radix(s, 10));

    let u32_parser = map_res(digit1, str::parse);
    let u32_parser2 = map_res(digit1, str::parse);
    terminated(
      map(
        tuple((
          terminated(parser_usize, tag(":")),
          preceded(
            space1,
            alt((
              map(tag("Buy"), |_| (OrderType::Buy)),
              map(tag("Sell"), |_| (OrderType::Sell)),
            )),
          ),
          preceded(space1, u32_parser),
          terminated(preceded(tag(" BTC @ "), u32_parser2), tag(" USD")),
        )),
        |(a, b, c, d)| Order {
          id: a,
          order_type: b,
          price: d,
          quantity: c,
        },
      ),
      newline,
    )(input)
  }

  pub fn parse_order_input(input : &String) -> Option<Order> {
    parse_order(input).ok().map(|x| x.1)
  }
  

  #[cfg(test)]
  pub mod order_parsing_tests {
 
    pub use crate::orderparser::{parse_order};
    use crate::matcher::matcheri::{
        Order,
        OrderType::{Buy, Sell},
      };
   
    #[test]
    fn test_sell_order_parsing() {      
      let t = Order {
        id: 1,
        order_type: Sell,
        price: 5000,
        quantity: 100,
      };
  
      assert_eq!(parse_order("1: Sell 100 BTC @ 5000 USD\n"), Ok(("", t)));
    }

    #[test]
    fn test_buy_order_parsing() {    
      let t1 = Order {
        id: 2,
        order_type: Buy,
        price: 5001,
        quantity: 50,
      };
      assert_eq!(parse_order("2: Buy 50 BTC @ 5001 USD\n"), Ok(("", t1)));
    }
  }