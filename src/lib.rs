// Cadence - An extensible Statsd client for Rust!
//
// Copyright 2015-2016 TSH Labs
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


//! An extensible Statsd client for Rust!
//!
//! [Statsd](https://github.com/etsy/statsd) is a network server that listens for
//! metrics (things like counters and timers) sent over UDP and sends aggregates of
//! these metrics to a backend service of some kind (often
//! [Graphite](http://graphite.readthedocs.org/)).
//!
//! Cadence is a client written in Rust for interacting with a Statsd server. You
//! might want to emit metrics (using Cadence, sending them to a Statsd server) in
//! your Rust server application.
//!
//! For example, if you are running a Rust web service you might want to record:
//!
//! * Number of succesful requests
//! * Number of error requests
//! * Time taken for each request
//!
//! Cadence is a flexible and easy way to do this!
//!
//! ## Features
//!
//! * Support for emitting counters, timers, gauges, and meters to Statsd over UDP.
//! * Support for alternate backends via the `MetricSink` trait.
//! * A simple yet flexible API for sending metrics.
//!
//! ## Install
//!
//! To make use of Cadence in your project, add it as a dependency in your `Cargo.toml`
//! file.
//!
//! ``` toml
//! [dependencies]
//! cadence = "x.y.z"
//! ```
//!
//! Then, link to it in your library or application.
//!
//! ``` rust,no_run
//! // bin.rs or lib.rs
//! extern crate cadence;
//!
//! // rest of your library or application
//! ```
//!
//! ## Usage
//!
//! Some examples of how to use Cadence are shown below.
//!
//! ### Simple Use
//!
//! Simple usage of Cadence is shown below. In this example, we just import
//! the client, create an instance that will write to some imaginary metrics
//! server, and send a few metrics.
//!
//! ``` rust,no_run
//! // Import the client.
//! use cadence::prelude::*;
//! use cadence::{StatsdClient, UdpMetricSink, DEFAULT_PORT};
//!
//! // Create client that will write to the given host over UDP.
//! //
//! // Note that you'll probably want to actually handle any errors creating
//! // the client when you use it for real in your application. We're just
//! // using .unwrap() here since this is an example!
//! let host = ("metrics.example.com", DEFAULT_PORT);
//! let client = StatsdClient::<UdpMetricSink>::from_udp_host(
//!     "my.metrics", host).unwrap();
//!
//! // Emit metrics!
//! client.incr("some.counter");
//! client.time("some.methodCall", 42);
//! client.gauge("some.thing", 7);
//! client.meter("some.value", 5);
//! ```
//!
//! ### Buffered UDP Sink
//!
//! While sending a metric over UDP is very fast, the overhead of frequent
//! network calls can start to add up. This is especially true if you are
//! writing a high performance application that emits a lot of metrics.
//!
//! To make sure that metrics aren't interfering with the performance of
//! your application, you may want to use a `MetricSink` implentation that
//! buffers multiple metrics before sending them in a single network
//! operation. For this, there's `BufferedUdpMetricSink`. An example of
//! using this sink is given below.
//!
//! ``` rust,no_run
//! use std::net::UdpSocket;
//! use cadence::prelude::*;
//! use cadence::{StatsdClient, BufferedUdpMetricSink, DEFAULT_PORT};
//!
//! let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
//! socket.set_nonblocking(true).unwrap();
//!
//! let host = ("metrics.example.com", DEFAULT_PORT);
//! let sink = BufferedUdpMetricSink::from(host, socket).unwrap();
//! let client = StatsdClient::from_sink("my.prefix", sink);
//!
//! client.count("my.counter.thing", 29);
//! client.time("my.service.call", 214);
//! client.incr("some.event");
//! ```
//!
//! As you can see, using this buffered UDP sink is no more complicated
//! than using the regular, non-buffered, UDP sink.
//!
//! The only downside to this sink is that metrics aren't written to the
//! Statsd server until the buffer is full. If you have a busy application
//! that is constantly emitting metrics, this shouldn't be a problem.
//! However, if your application only occasionally emits metrics, this sink
//! might result in the metrics being delayed for a little while until the
//! buffer fills.
//!
//! ### Asynchronous Metric Sink
//!
//! To make sure emitting metrics doesn't interfere with the performance
//! of your application (even though emitting metrics is generally quite
//! fast), it's probably a good idea to make sure metrics are emitted in
//! in a different thread than your application thread.
//!
//! To allow you do this, there is `AsyncMetricSink`. This sink allows you
//! to wrap any other metric sink and send metrics using a thread pool,
//! asynchronously from the flow of your application.
//!
//! The requirements for the wrapped metric sink are that it is thread
//! safe, meaning that it implements the `Send` and `Sync` traits.
//! Additionally, the wrapped sink should implement the `Clone` trait since
//! this is how the `AsyncMetricSink` is designed to be shared between
//! threads (see the source code for the `AsyncMetricSink` for more info).
//! If you're using the `AsyncMetricSink` with another sink from
//! Cadence, you don't need to worry: they are all thread safe and implement
//! the `Clone` trait.
//!
//! An example of using the `AsyncMetricSink` to wrap a buffered UDP
//! metric sink is given below.
//!
//! ``` rust,no_run
//! use std::net::UdpSocket;
//! use cadence::prelude::*;
//! use cadence::{StatsdClient, AsyncMetricSink, BufferedUdpMetricSink,
//!               DEFAULT_PORT};
//!
//! let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
//! socket.set_nonblocking(true).unwrap();
//!
//! let host = ("metrics.example.com", DEFAULT_PORT);
//! let udp_sink = BufferedUdpMetricSink::from(host, socket).unwrap();
//! let async_sink = AsyncMetricSink::from(udp_sink);
//! let client = StatsdClient::from_sink("my.prefix", async_sink);
//!
//! client.count("my.counter.thing", 29);
//! client.time("my.service.call", 214);
//! client.incr("some.event");
//! ```
//!
//! ### Counted, Timed, Gauged, Metered, and MetricClient Traits
//!
//! Each of the methods that the Cadence `StatsdClient` struct uses to send
//! metrics are implemented as a trait. There is also a trait that combines
//! all of these other traits. If we want, we can just use one of the trait
//! types to refer to the client instance. This might be useful to you if
//! you'd like to swap out the actual Cadence client with a dummy version
//! when you are unit testing your code or want to abstract away all the
//! implementation details of the client being used behind a trait and
//! pointer.
//!
//! Each of these traits are exported in the prelude module. They are also
//! available in the main module but aren't typically used like that.
//!
//! ``` rust,no_run
//! use cadence::prelude::*;
//! use cadence::{StatsdClient, UdpMetricSink, DEFAULT_PORT};
//!
//!
//! pub struct User {
//!     id: u64,
//!     username: String,
//!     email: String
//! }
//!
//!
//! // Here's a simple DAO (Data Access Object) that doesn't do anything but
//! // uses a metric client to keep track of the number of times the
//! // 'getUserById' method gets called.
//! pub struct MyUserDao {
//!     metrics: Box<MetricClient>
//! }
//!
//!
//! impl MyUserDao {
//!     // Create a new instance that will use the StatsdClient
//!     pub fn new<T: MetricClient + 'static>(metrics: T) -> MyUserDao {
//!         MyUserDao { metrics: Box::new(metrics) }
//!     }
//!
//!     /// Get a new user by their ID
//!     pub fn get_user_by_id(&self, id: u64) -> Option<User> {
//!         self.metrics.incr("getUserById");
//!         None
//!     }
//! }
//!
//!
//! // Create a new Statsd client that writes to "metrics.example.com"
//! let host = ("metrics.example.com", DEFAULT_PORT);
//! let metrics = StatsdClient::<UdpMetricSink>::from_udp_host(
//!     "counter.example", host).unwrap();
//!
//! // Create a new instance of the DAO that will use the client
//! let dao = MyUserDao::new(metrics);
//!
//! // Try to lookup a user by ID!
//! match dao.get_user_by_id(123) {
//!     Some(u) => println!("Found a user!"),
//!     None => println!("No user!")
//! };
//! ```
//!
//! ### Custom Metric Sinks
//!
//! The Cadence `StatsdClient` uses implementations of the `MetricSink`
//! trait to send metrics to a metric server. Most users of the Candence
//! library probably want to use the `AsyncMetricSink` wrapping an instance
//! of the `BufferedMetricSink`.
//!
//! However, maybe you want to do something not covered by an existing sink.
//! An example of creating a custom sink is below.
//!
//! ``` rust,no_run
//! use std::io;
//! use cadence::prelude::*;
//! use cadence::{StatsdClient, MetricSink, DEFAULT_PORT};
//!
//! pub struct MyMetricSink;
//!
//!
//! impl MetricSink for MyMetricSink {
//!     fn emit(&self, metric: &str) -> io::Result<usize> {
//!         // Your custom metric sink implementation goes here!
//!         Ok(0)
//!     }
//! }
//!
//!
//! let sink = MyMetricSink;
//! let client = StatsdClient::from_sink("my.prefix", sink);
//!
//! client.count("my.counter.thing", 42);
//! client.time("my.method.time", 25);
//! client.incr("some.other.counter");
//! ```
//!
//! ### Custom UDP Socket
//!
//! Most users of the Cadence `StatsdClient` will be using it to send metrics
//! over a UDP socket. If you need to customize the socket, for example you
//! want to use the socket in blocking mode but set a write timeout, you can
//! do that as demonstrated below.
//!
//! ``` rust,no_run
//! use std::net::UdpSocket;
//! use std::time::Duration;
//! use cadence::prelude::*;
//! use cadence::{StatsdClient, UdpMetricSink, DEFAULT_PORT};
//!
//! let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
//! socket.set_write_timeout(Some(Duration::from_millis(1))).unwrap();
//!
//! let host = ("metrics.example.com", DEFAULT_PORT);
//! let sink = UdpMetricSink::from(host, socket).unwrap();
//! let client = StatsdClient::from_sink("my.prefix", sink);
//!
//! client.count("my.counter.thing", 29);
//! client.time("my.service.call", 214);
//! client.incr("some.event");
//! ```
//!


#[macro_use]
extern crate log;
extern crate threadpool;


pub const DEFAULT_PORT: u16 = 8125;


pub use self::client::{Counted, Timed, Gauged, Metered, MetricClient,
                       StatsdClient};


pub use self::sinks::{MetricSink, ConsoleMetricSink, LoggingMetricSink,
                      NopMetricSink, UdpMetricSink, BufferedUdpMetricSink,
                      AsyncMetricSink};


pub use self::types::{MetricResult, MetricError, ErrorKind, Counter, Timer,
                      Gauge, Meter};


pub mod prelude;
mod client;
mod io;
mod sinks;
mod types;
