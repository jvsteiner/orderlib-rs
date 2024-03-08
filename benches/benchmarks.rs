#[macro_use]
extern crate criterion;
use criterion::{black_box, Criterion};
use orderlib::orderlib::Order;
use orderlib::orderlib::OrderBook;
use orderlib::orderlib::OrderSide::Buy;
use orderlib::orderlib::OrderSide::Sell;
use orderlib::orderlib::OrderType::Limit;

pub fn my_benchmark(c: &mut Criterion) {
    let mut order_book: OrderBook = OrderBook::new();
    c.bench_function("symmetric_buy_sell", |b| {
        b.iter(|| {
            order_book.add(Order::new(Buy, 20, 100, Limit));
            order_book.add(Order::new(Buy, 20, 101, Limit));
            order_book.add(Order::new(Sell, 40, 100, Limit));
        })
    });
}

criterion_group!(benches, my_benchmark);
criterion_main!(benches);
