#[macro_use]
extern crate criterion;
use criterion::{black_box, Criterion};
use orderlib::orderlib::{Order, OrderBook, OrderSide::Buy, OrderSide::Sell, OrderType::Limit};

pub fn symmetric_buy_sell(c: &mut Criterion) {
    let mut order_book: OrderBook = OrderBook::new();
    c.bench_function("symmetric_buy_sell", |b| {
        b.iter(|| {
            order_book.add(Order::new(Buy, 20, 100, Limit));
            order_book.add(Order::new(Buy, 20, 101, Limit));
            order_book.add(Order::new(Sell, 40, 100, Limit));
        })
    });
}

criterion_group!(benches, symmetric_buy_sell);
criterion_main!(benches);
