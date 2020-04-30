use clap::{App, Arg};
use log::error;
use metric::LogCompactionInMemoryMetrics;
use metric::MessageMetrics;
use prettytable::{row, Table};
use prettytable::Cell;
use prettytable::Row;
use std::cmp::max;
use std::collections::HashMap;
use std::process::exit;
use std::time::Instant;
#[macro_use] extern crate prettytable;

mod kafka;
mod metric;
mod fnv32;

fn main() {
    let matches = App::new("Kafka Topic Analyzer")
        .bin_name("kafka-topic-analyzer")
        
        .version("0.4.1")

        .arg(Arg::with_name("topic")
            .short('t')
            .long("topic")
            .value_name("TOPIC")
            .about("The topic to analyze")
            .takes_value(true)
            .required(true)
        )
        .arg(Arg::with_name("max-records")
            .short('r')
            .long("max-records")
            .value_name("MAX_RECORDS")
            .about("The number of consumed records")
            .takes_value(true)
            .required(false)
        )
        .arg(Arg::with_name("bootstrap-server")
            .short('b')
            .long("bootstrap-server")
            .value_name("BOOTSTRAP_SERVER")
            .about("Bootstrap server(s) to work with, comma separated")
            .takes_value(true)
            .required(true)
        )
        .arg(Arg::with_name("count-alive-keys")
            .short('c')
            .long("count-alive-keys")
            .about("Counts the effective number of alive keys in a log compacted topic by saving the \
            state for each key in a local file and counting the result at the end of the read operation.\
            A key is 'alive' when it is present and has a non-null value in it's latest-offset version")
            .required(false))
        .get_matches();

    let start_time = Instant::now();

    let mut partitions = Vec::<i32>::new();
    let start_offsets: HashMap<i32, i64>;
    let end_offsets: HashMap<i32, i64>;
    let topic = matches.value_of("topic").unwrap();
    let bootstrap_server = matches.value_of("bootstrap-server").unwrap();
    let max_records: u64 = matches.value_of_t("max-records").unwrap_or(0);

    let mut log_compaction_metrics = match matches.occurrences_of("count-alive-keys") {
        1 => Some(LogCompactionInMemoryMetrics::new()),
        _ => None
    };

    let mut metrics = MessageMetrics::new();
    {
        let mut topic_analyzer = kafka::TopicAnalyzer::new_from_bootstrap_servers(bootstrap_server, max_records);
        let offsets = topic_analyzer.get_topic_offsets(topic);
        start_offsets = offsets.0;
        end_offsets = offsets.1;

        if end_offsets.values().all(|v| *v == 0i64) {
            error!("Given topic has no content, no analysis possible. Exiting.");
            exit(-2)
        }

        for v in start_offsets.keys() {
            partitions.push(*v);
        }
        partitions.sort();

        topic_analyzer.add_metric_handler(&mut metrics);

        match log_compaction_metrics.as_mut() {
            Some(l) => {
                topic_analyzer.add_metric_handler(l);
            }
            None => {}
        }

        topic_analyzer.read_topic_into_metrics(topic, &end_offsets);
    }


    let duration_secs = start_time.elapsed().as_secs();

    {
        let metrics_cloned = &metrics;
        println!();
        println!("{}", "=".repeat(120));
        println!("Calculating statistics...");
        println!("Topic {}", topic);
        println!("Scanning took: {} seconds", duration_secs);
        println!("Estimated Msg/s: {}", (metrics_cloned.overall_count() / max(duration_secs, 1)));
        println!("{}", "-".repeat(120));
        println!("Earliest Message: {}", metrics_cloned.earliest_message());
        println!("Latest Message: {}", metrics_cloned.latest_message());
        println!("{}", "-".repeat(120));
        println!("Largest Message: {} bytes", metrics_cloned.largest_message());
        println!("Smallest Message: {} bytes", metrics_cloned.smallest_message());
        println!("Topic Size: {} bytes", metrics_cloned.overall_size());

        match log_compaction_metrics {
            Some(l) => {
                println!("{}", "-".repeat(120));
                println!("Alive keys: {}", l.sum_all_alive());
                println!("{}", "-".repeat(120));
            }
            None => {}
        }

        println!("{}", "=".repeat(120));
        let mut table = Table::new();
        table.add_row(row!["P", "< OS", "> OS", "Total", "Alive", "Tmb", "DR", "K Null", "K !Null", "P-Bytes", "K-Bytes", "V-Bytes", "A K-Sz", "A V-Sz", "A M-Sz"]);

        for partition in partitions {
            let key_size_avg = metrics.key_size_avg(partition);
            table.add_row(Row::new(vec![
                Cell::new(format!("{}", partition).as_str()),
                Cell::new(format!("{}", &start_offsets[&partition]).as_str()),
                Cell::new(format!("{}", &end_offsets[&partition]).as_str()),
                Cell::new(format!("{}", metrics_cloned.total(partition)).as_str()),
                Cell::new(format!("{}", metrics_cloned.alive(partition)).as_str()),
                Cell::new(format!("{}", metrics_cloned.tombstones(partition)).as_str()),
                Cell::new(format!("{0:.4}", metrics_cloned.dirty_ratio(partition)).as_str()),
                Cell::new(format!("{}", metrics_cloned.key_null(partition)).as_str()),
                Cell::new(format!("{}", metrics_cloned.key_non_null(partition)).as_str()),
                Cell::new(format!("{}", metrics_cloned.key_size_sum(partition) + metrics_cloned.value_size_sum(partition)).as_str()),
                Cell::new(format!("{}", metrics_cloned.key_size_sum(partition)).as_str()),
                Cell::new(format!("{}", metrics_cloned.value_size_sum(partition)).as_str()),
                Cell::new(format!("{}", key_size_avg).as_str()),
                Cell::new(format!("{}", metrics_cloned.value_size_avg(partition)).as_str()),
                Cell::new(format!("{}", metrics_cloned.message_size_avg(partition)).as_str()),
            ]));
        }

        println!("| K = Key, V = Value, P = Partition, Tmb = Tombstone(s), Sz = Size");
        println!("| DR = Dirty Ratio, A = Average, Lst = last, < OS = start offset, > OS = end offset");
        table.printstd();
        println!();
        println!("{}", "=".repeat(120));
    }
}
