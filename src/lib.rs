#![crate_name = "orderlib"]

pub mod orderlib {
    /// orderlib is a package that provides trading logic and order primitives for
    /// use in a provided, high performance data structure.  A std::collections::BTreeSet
    /// is used to hold orders.  Orders are processed in Price/Time priority.
    use std::cmp;
    use std::cmp::Ordering;
    use std::collections::BTreeSet;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum OrderType {
        /// A Type to represent the orders that traders want to make.
        /// Fill only up to the price limit indicated
        Limit,
        /// Fill all, ignores price field.
        Market,
        /// Check if it can be fully filled, execute if so, cancel otherwise
        Fok,
        /// Fill as much as possible, cancel the rest
        Ioc,
        /// Do nothing until the entire order can be filled at the limit price or better, then execute
        Aon,
    }

    #[derive(Clone, Copy, Debug, PartialEq)]
    pub enum OrderSide {
        /// Side represents whether the order means to sell as asset or to buy it.
        /// This is useful for ensuring the order ends up in the right place.
        Buy,
        Sell,
    }

    #[derive(Clone, Copy, Debug)]
    pub struct Order {
        pub order_id: i64,
        pub order_number: i64,
        pub order_side: OrderSide,
        pub size: i64,
        pub price: i64,
        pub timestamp: i64,
        pub order_type: OrderType,
        // user: &'user User<'user>, // this is a reference to the user who placed the order - not used
    }

    impl Order {
        pub fn new(order_side: OrderSide, size: i64, price: i64, order_type: OrderType) -> Order {
            Order {
                order_id: 0,
                order_number: 0,
                order_side,
                size,
                price,
                timestamp: 0,
                order_type,
            }
        }
    }

    impl PartialEq for Order {
        fn eq(&self, other: &Self) -> bool {
            self.order_number == other.order_number
        }
    }

    impl Eq for Order {}

    impl PartialOrd for Order {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
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

    #[derive(Debug)]
    pub struct Fill {
        pub size: i64,
        pub price: i64,
        pub direction: OrderSide,
        pub aggressor_id: i64,
        pub passive_id: i64,
        pub timestamp: i64,
        pub fill_id: i64,
    }

    #[derive(Debug, PartialEq)]
    pub struct LimitReport {
        pub price: f64,
        pub size: i64,
    }

    pub fn get_epoch_ms() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }

    #[derive(Debug)]
    pub struct OrderBook {
        // will be in increasing order of price, best is last
        buy_orders: BTreeSet<Order>,
        sell_orders: BTreeSet<Order>,
        counter: i64,
    }

    impl OrderBook {
        /// Constructs a new `OrderBook`.
        ///
        /// # Examples
        ///
        /// ```
        /// use orderlib::new;
        ///
        /// let ob = orderlib::new();
        /// ```
        pub fn new() -> OrderBook {
            OrderBook {
                buy_orders: BTreeSet::new(),
                sell_orders: BTreeSet::new(),
                counter: 1230,
            }
        }

        pub fn next(&mut self, side: OrderSide) -> Option<&Order> {
            match side {
                OrderSide::Buy => {
                    return self.sell_orders.first();
                }
                OrderSide::Sell => {
                    return self.buy_orders.first();
                }
            }
        }

        pub fn add(&mut self, mut order: Order) -> (i64, Vec<Fill>) {
            order.timestamp = get_epoch_ms();
            order.order_number = self.counter;
            self.counter += 1;
            match order.order_type {
                OrderType::Fok => {}
                OrderType::Aon => {}
                _ => {}
            }
            match order.order_side {
                OrderSide::Buy => {
                    return (order.order_number, self.trade(order, 1));
                }
                OrderSide::Sell => {
                    return (order.order_number, self.trade(order, -1));
                }
            }
            // fills
        }

        pub fn remove(&mut self, mut order: Order) -> bool {
            match order.order_side {
                OrderSide::Buy => {
                    order.price = -order.price;
                    return self.buy_orders.remove(&order);
                }
                OrderSide::Sell => {
                    return self.sell_orders.remove(&order);
                }
            }
        }

        pub fn replace(&mut self, order: Order) -> Option<Order> {
            match order.order_side {
                OrderSide::Buy => {
                    return self.buy_orders.replace(order);
                }
                OrderSide::Sell => {
                    return self.sell_orders.replace(order);
                }
            }
        }

        pub fn best_bid(&self) -> Option<Order> {
            let mut bid: Order = self.buy_orders.first().cloned().unwrap();
            bid.price = -1 * bid.price;
            Some(bid)
        }

        pub fn best_offer(&self) -> Option<Order> {
            self.sell_orders.first().cloned()
        }

        pub fn len_bids(&self) -> usize {
            self.buy_orders.len()
        }

        pub fn len_offers(&self) -> usize {
            self.sell_orders.len()
        }

        pub fn size_at_limit(&self, direction: OrderSide, mut price: f64) -> Option<LimitReport> {
            let opposite_stack: &BTreeSet<Order>;
            let mut found_size: i64 = 0;
            let mut size_weighted_price: i64 = 0;
            match direction {
                OrderSide::Sell => {
                    price = -price;
                    opposite_stack = &self.buy_orders;
                }
                OrderSide::Buy => {
                    opposite_stack = &self.sell_orders;
                }
            }
            if opposite_stack.len() == 0 {
                return None;
            }

            for order in opposite_stack.iter() {
                let add_these = could_add(size_weighted_price, found_size, *order, price);
                if add_these <= 0 {
                    break;
                }
                found_size += add_these;
                size_weighted_price += add_these * order.price;
            }

            if found_size == 0 {
                return None;
            }
            let limit_report: LimitReport = LimitReport {
                price: size_weighted_price.abs() as f64 / found_size as f64,
                size: found_size,
            };
            Some(limit_report)
        }

        pub fn limit_at_size(&self, direction: OrderSide, size: i64) -> Option<LimitReport> {
            let mut unfound_size: i64 = size;
            let mut size_weighted_price: i64 = 0;
            let opposite_stack: &BTreeSet<Order>;
            match direction {
                OrderSide::Sell => {
                    opposite_stack = &self.buy_orders;
                }
                OrderSide::Buy => {
                    opposite_stack = &self.sell_orders;
                }
            }
            for order in opposite_stack.iter() {
                if unfound_size <= order.size {
                    size_weighted_price += unfound_size * order.price;
                    unfound_size = 0;
                    break;
                } else {
                    size_weighted_price += order.size * order.price;
                    unfound_size -= order.size;
                }
            }
            if size == 0 {
                return None;
            } else {
                let found_size: i64 = size - unfound_size;
                let limit_report: LimitReport = LimitReport {
                    price: size_weighted_price.abs() as f64 / found_size as f64,
                    size: found_size,
                };
                Some(limit_report)
            }
        }

        fn trade(&mut self, mut order: Order, bs: i64) -> Vec<Fill> {
            let mut fills: Vec<Fill> = Vec::new();
            order.price = bs * order.price;

            let opp: &mut BTreeSet<Order>;
            let these: &mut BTreeSet<Order>;

            match order.order_side {
                OrderSide::Buy => {
                    opp = &mut self.sell_orders;
                    these = &mut self.buy_orders;
                }
                OrderSide::Sell => {
                    opp = &mut self.buy_orders;
                    these = &mut self.sell_orders;
                }
            }

            while opp.len() > 0 && order.size > 0 {
                let next_order: &Order = opp.first().unwrap();

                if next_order.price > order.price && order.order_type != OrderType::Market {
                    break;
                }
                let mut fill: Fill = Fill {
                    size: 0,
                    price: bs * next_order.price,
                    direction: order.order_side,
                    aggressor_id: order.order_id,
                    passive_id: next_order.order_id,
                    timestamp: get_epoch_ms(),
                    fill_id: 0,
                };
                if order.size < next_order.size {
                    fill.size = order.size;
                    let mut next_order_clone: Order = next_order.clone();
                    next_order_clone.size -= order.size;
                    // This copy and replace should be unnecessary
                    let replacement: Order = next_order_clone;
                    opp.replace(replacement);
                    order.size = 0;
                    fills.push(fill);
                    break;
                } else if order.size > next_order.size {
                    fill.size = next_order.size;
                    let next_order_clone: Order = next_order.clone();
                    order.size -= next_order.size;
                    opp.remove(&next_order_clone);
                    fills.push(fill);
                } else if order.size == next_order.size {
                    fill.size = next_order.size;
                    let next_order_clone: Order = next_order.clone();
                    order.size = 0;
                    opp.remove(&next_order_clone);
                    fills.push(fill);
                    break;
                }
            }

            if order.size > 0 && order.order_type != OrderType::Ioc {
                order.price = -1 * order.price;
                these.replace(order);
            }
            fills
        }
    }

    fn could_add(size_weighted_price: i64, found_size: i64, order: Order, lim: f64) -> i64 {
        let oprice = order.price;
        let denom = lim - oprice as f64;
        // switched this from <= to >=
        if denom >= 0.0 {
            order.size
        } else {
            cmp::min(
                order.size,
                cmp::max(
                    0,
                    ((size_weighted_price as f64 - lim * found_size as f64) / denom).trunc() as i64,
                ),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::orderlib::Fill;
    use super::orderlib::LimitReport;
    use super::orderlib::Order;
    use super::orderlib::OrderBook;
    use super::orderlib::OrderSide;
    use super::orderlib::OrderSide::Buy;
    use super::orderlib::OrderSide::Sell;
    use super::orderlib::OrderType::Ioc;
    use super::orderlib::OrderType::Limit;
    use super::orderlib::OrderType::Market;

    #[test]
    fn test_add_delete_orderbook() {
        let mut order_book: OrderBook = OrderBook::new();
        let mut order1 = Order::new(Buy, 20, 100, Limit);
        let order1num = order_book.add(order1).0; // len == 1
        assert_eq!(order_book.len_bids(), 1);
        order_book.remove(order1);
        assert_eq!(order_book.len_bids(), 1); // doesn't work, len still == 1
        let original: Order = order_book.best_bid().unwrap();
        order1.order_number = order1num;
        order_book.remove(order1);
        assert_eq!(order_book.len_bids(), 0); // should now work
        order_book.remove(original);
        assert_eq!(order_book.len_bids(), 0); // doesn't work, already removed above

        order_book.add(Order::new(Buy, 20, 100, Limit));
        order_book.add(Order::new(Buy, 30, 101, Limit));
        assert_eq!(order_book.len_bids(), 2);
        let first: Order = order_book.best_bid().unwrap();
        assert_eq!(first.price, 101);
        assert!(order_book.remove(first));
        assert_eq!(order_book.len_bids(), 1);
        let last: Order = order_book.best_bid().unwrap();
        assert_eq!(last.price, 100);
    }

    #[test]
    fn test_sell_limit_order() {
        let mut order_book: OrderBook = OrderBook::new();
        order_book.add(Order::new(Buy, 20, 100, Limit));
        assert_eq!(order_book.len_bids(), 1);
        assert_eq!(order_book.len_offers(), 0);
        order_book.add(Order::new(Buy, 20, 101, Limit));
        let fills: Vec<Fill> = order_book
            .add(Order::new(OrderSide::Sell, 31, 101, Limit))
            .1;
        assert_eq!(fills.len(), 1);
        assert_eq!(fills[0].size, 20);
        assert_eq!(fills[0].price, 101);
        assert_eq!(order_book.len_bids(), 1);
        assert_eq!(order_book.len_offers(), 1);
        assert_eq!(order_book.best_bid().unwrap().price, 100);
        assert_eq!(order_book.best_bid().unwrap().size, 20);
        assert_eq!(order_book.best_offer().unwrap().price, 101);
        assert_eq!(order_book.best_offer().unwrap().size, 11);
    }

    #[test]
    fn test_buy_limit_order() {
        let mut order_book: OrderBook = OrderBook::new();
        order_book.add(Order::new(Sell, 20, 100, Limit));
        assert_eq!(order_book.len_bids(), 0);
        assert_eq!(order_book.len_offers(), 1);
        order_book.add(Order::new(Sell, 20, 101, Limit));
        assert_eq!(order_book.best_offer().unwrap().price, 100);
        let fills: Vec<Fill> = order_book.add(Order::new(Buy, 31, 100, Limit)).1;
        assert_eq!(fills.len(), 1);
        assert_eq!(fills[0].size, 20);
        assert_eq!(fills[0].price, 100);
        assert_eq!(order_book.len_bids(), 1);
        assert_eq!(order_book.len_offers(), 1);
        assert_eq!(order_book.best_bid().unwrap().price, 100);
        assert_eq!(order_book.best_bid().unwrap().size, 11);
        assert_eq!(order_book.best_offer().unwrap().price, 101);
        assert_eq!(order_book.best_offer().unwrap().size, 20);
    }

    #[test]
    fn test_timestamps() {
        let mut order_book: OrderBook = OrderBook::new();
        let order: Order = Order::new(Sell, 20, 100, Limit);
        assert_eq!(order.timestamp, 0);
        order_book.add(order);
        let first: Order = order_book.best_offer().unwrap();
        assert_ne!(first.timestamp, 0);
        let fills: Vec<Fill> = order_book.add(Order::new(Buy, 31, 100, Limit)).1;
        assert_eq!(fills.len(), 1);
        assert_ne!(fills[0].timestamp, 0);
    }

    #[test]
    fn test_delete_orders() {
        let mut order_book: OrderBook = OrderBook::new();
        let order_1_number = order_book.add(Order::new(Buy, 20, 100, Limit)).0;
        order_book.add(Order::new(Buy, 20, 101, Limit));
        let order_3_number = order_book.add(Order::new(Sell, 20, 102, Limit)).0;
        order_book.add(Order::new(Sell, 20, 103, Limit));
        assert_eq!(order_book.len_bids(), 2);
        assert_eq!(order_book.len_offers(), 2);
        let mut to_delete_order_1: Order = Order::new(Buy, 20, 100, Limit);
        to_delete_order_1.order_number = order_1_number;
        order_book.remove(to_delete_order_1);
        assert_eq!(order_book.len_bids(), 1);
        assert_eq!(order_book.len_offers(), 2);
        let mut order3_copy = Order::new(Sell, 20, 102, Limit);
        order3_copy.order_number = order_3_number;
        order_book.remove(order3_copy);
        assert_eq!(order_book.len_bids(), 1);
        assert_eq!(order_book.len_offers(), 1);
    }

    #[test]
    fn test_sell_market_order() {
        let mut order_book: OrderBook = OrderBook::new();
        order_book.add(Order::new(Buy, 20, 100, Limit));
        order_book.add(Order::new(Buy, 20, 101, Limit));
        let fills: Vec<Fill> = order_book.add(Order::new(Sell, 31, 103, Market)).1;
        assert_eq!(fills.len(), 2);
        assert_eq!(order_book.best_bid().unwrap().size, 9);
        assert_eq!(fills[0].size, 20);
        assert_eq!(fills[0].price, 101);
        assert_eq!(fills[1].size, 11);
        assert_eq!(fills[1].price, 100);
    }

    #[test]
    fn test_sell_ioc_order() {
        let mut order_book: OrderBook = OrderBook::new();
        order_book.add(Order::new(Buy, 20, 100, Limit));
        order_book.add(Order::new(Buy, 20, 101, Limit));
        let fills: Vec<Fill> = order_book.add(Order::new(Sell, 31, 101, Ioc)).1;
        assert_eq!(fills.len(), 1);
        assert_eq!(order_book.best_offer(), None);
        assert_eq!(fills[0].size, 20);
        assert_eq!(fills[0].price, 101);
    }

    #[test]
    fn test_limit_at_size_report() {
        let mut order_book: OrderBook = OrderBook::new();
        order_book.add(Order::new(Buy, 20, 100, Limit));
        order_book.add(Order::new(Buy, 20, 101, Limit));
        order_book.add(Order::new(Sell, 11, 102, Limit));
        let mut report = order_book.limit_at_size(Sell, 30).unwrap();
        assert_eq!(
            LimitReport {
                price: 100.0 + 2.0 / 3.0,
                size: 30,
            },
            report
        );
        let new_order = Order::new(Sell, 31, 101, Limit);
        let fills = order_book.add(new_order).1;
        report = order_book.limit_at_size(Sell, 30).unwrap();
        assert_eq!(report.price, 100.0);
        assert_eq!(report.size, 20);
        report = order_book.limit_at_size(Buy, 30).unwrap();
        assert_eq!(report.price, 101.5);
        assert_eq!(report.size, 22);
        assert_eq!(order_book.best_offer().unwrap().size, 11);
        assert_eq!(fills[0].size, 20);
        assert_eq!(fills[0].price, 101);
    }

    #[test]
    fn test_size_at_limit_report() {
        let mut order_book: OrderBook = OrderBook::new();
        order_book.add(Order::new(Buy, 20, 100, Limit));
        order_book.add(Order::new(Buy, 20, 101, Limit));
        order_book.add(Order::new(Sell, 20, 102, Limit));
        order_book.add(Order::new(Sell, 20, 103, Limit));
        let mut report = order_book.size_at_limit(Sell, 100.5).unwrap();
        assert_eq!(report.price, 100.5);
        assert_eq!(report.size, 40);
        report = order_book.size_at_limit(Sell, 100.0).unwrap();
        assert_eq!(report.price, 100.5);
        assert_eq!(report.size, 40);
        report = order_book.size_at_limit(Sell, 100.75).unwrap();
        assert_eq!(report.price, 100.76923076923077);
        assert_eq!(report.size, 26);
        report = order_book.size_at_limit(Buy, 102.5).unwrap();
        assert_eq!(report.price, 102.5);
        assert_eq!(report.size, 40);
        assert_eq!(order_book.size_at_limit(Buy, 100.5), None);
    }

    #[test]
    fn test_no_trade() {
        let mut order_book: OrderBook = OrderBook::new();
        order_book.add(Order::new(Buy, 20, 100, Limit));
        order_book.add(Order::new(Buy, 20, 101, Limit));
        let fills = order_book.add(Order::new(Sell, 31, 102, Limit)).1;
        assert_eq!(order_book.best_offer().unwrap().size, 31);
        assert_eq!(fills.len(), 0);
    }
}
