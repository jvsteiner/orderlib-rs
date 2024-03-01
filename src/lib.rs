use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, PartialEq)]
enum OrderType {
    Limit,
    Market,
    Fok,
    Ioc,
    Aon,
}

#[derive(Clone, Copy, PartialEq)]
enum OrderSide {
    Buy,
    Sell,
}

#[derive(Clone, Copy)]
struct Order {
    order_id: u64,
    order_number: u64,
    order_side: OrderSide,
    size: u64,
    price: i64,
    timestamp: u64,
    order_type: OrderType,
    // user: &'user User<'user>, // this is a reference to the user who placed the order - not used
}

impl Order {
    fn new(order_side: OrderSide, size: u64, price: i64) -> Order {
        Order {
            order_id: 0,
            order_number: 0,
            order_side,
            size,
            price,
            timestamp: 0,
            order_type: OrderType::Limit,
        }
    }
}

impl PartialEq for Order {
    fn eq(&self, other: &Self) -> bool {
        self.order_id == other.order_id
    }
}

impl Ord for Order {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.price > other.price {
            Ordering::Greater
        } else if self.price < other.price {
            Ordering::Less
        } else {
            if self.order_number < other.order_number {
                Ordering::Greater
            } else if self.order_number > other.order_number {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        }
    }
}

impl Eq for Order {}

impl PartialOrd for Order {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct Fill {
    size: u64,
    price: i64,
    direction: OrderSide,
    aggressor_id: u64,
    passive_id: u64,
    timestamp: u64,
    fill_id: u64,
}

struct LimitReport {
    price: f64,
    size: u64,
}

fn get_epoch_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

macro_rules! noop {
    () => {};
}
struct OrderBook {
    // will be in increasing order of price, best is last
    buy_orders: BTreeSet<Order>,
    sell_orders: BTreeSet<Order>,
    counter: u64,
}

impl OrderBook {
    fn new() -> OrderBook {
        OrderBook {
            buy_orders: BTreeSet::new(),
            sell_orders: BTreeSet::new(),
            counter: 0,
        }
    }

    fn add_order(&mut self, mut order: Order) -> Vec<Fill> {
        order.timestamp = get_epoch_ms() as u64;
        order.order_number = self.counter;
        self.counter += 1;
        let mut fills: Vec<Fill> = Vec::new();
        match order.order_type {
            OrderType::Fok => {
                noop!();
            }
            OrderType::Aon => {
                noop!();
            }
            _ => {
                noop!();
            }
        }
        match order.order_side {
            OrderSide::Buy => {
                fills = trade(order, &mut self.sell_orders, &mut self.buy_orders, -1);
            }
            OrderSide::Sell => {
                fills = trade(order, &mut self.buy_orders, &mut self.sell_orders, 1);
            }
        }
        fills
    }

    fn remove_order(&mut self, order: Order) {
        match order.order_side {
            OrderSide::Buy => {
                self.buy_orders.remove(&order);
            }
            OrderSide::Sell => {
                self.sell_orders.remove(&order);
            }
        }
    }

    fn replace_order(&mut self, order: Order) {
        match order.order_side {
            OrderSide::Buy => {
                self.buy_orders.replace(order);
            }
            OrderSide::Sell => {
                self.sell_orders.replace(order);
            }
        }
    }
}

fn trade(
    mut order: Order,
    opp: &mut BTreeSet<Order>,
    these: &mut BTreeSet<Order>,
    bs: i64,
) -> Vec<Fill> {
    let mut fills: Vec<Fill> = Vec::new();
    order.price = bs * order.price;

    while opp.len() > 0 && order.size > 0 {
        let mut next_order: &Order = opp.last().unwrap();
        if next_order.price > order.price {
            break;
        }
        let mut fill: Fill = Fill {
            size: 0,
            price: bs * next_order.price,
            direction: order.order_side,
            aggressor_id: order.order_id,
            passive_id: next_order.order_id,
            timestamp: get_epoch_ms() as u64,
            fill_id: 0,
        };
        if order.size < next_order.size {
            fill.size = order.size;
            let mut next_order_clone: Order = next_order.clone();
            next_order_clone.size -= order.size;
            let replacement: Order = next_order_clone;
            opp.replace(replacement);
            order.size = 0;
            fills.push(fill);
            break;
        } else if order.size > next_order.size {
            fill.size = next_order.size;
            let mut next_order_clone: Order = next_order.clone();
            order.size -= next_order.size;
            opp.remove(&next_order_clone);
            fills.push(fill);
        } else if order.size == next_order.size {
            fill.size = next_order.size;
            let mut next_order_clone: Order = next_order.clone();
            order.size = 0;
            opp.remove(&next_order_clone);
            fills.push(fill);
            break;
        }

        next_order = opp.last().unwrap();
    }

    if order.size > 0 && order.order_type != OrderType::Ioc {
        order.price = -1 * order.price;
        these.replace(order);
    }

    fills
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_add_delete_orderbook() {
        let mut order_book: crate::OrderBook = super::OrderBook::new();
        let mut order: super::Order = super::Order::new(super::OrderSide::Buy, 100, 100);
        let mut order2: super::Order = super::Order::new(super::OrderSide::Buy, 100, 101);
        order_book.add_order(order);
        order_book.add_order(order2);
        assert_eq!(order_book.buy_orders.len(), 2);
        let last: crate::Order = *order_book.buy_orders.last().unwrap();
        assert_eq!(last.price, 101);
        order_book.remove_order(last);
        let last: crate::Order = *order_book.buy_orders.last().unwrap();
        assert_eq!(order_book.buy_orders.len(), 1);
        assert_eq!(last.price, 100);
    }
}
