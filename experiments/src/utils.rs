use anyhow::{Context, Error};
use clap::{ArgEnum, Args};
use osmnodecache::{Advice, DenseFileCache};
use std::fmt::Debug;
use std::fmt::Write;
use std::ops::AddAssign;
use std::sync::mpsc::Receiver;
use std::thread;
use std::thread::JoinHandle;
use std::time::Instant;

use separator::Separatable;

#[derive(Debug)]
pub struct Histogram {
    data: Vec<usize>,
    data_sums: Vec<usize>,
    encoder: fn(usize) -> usize,
    formatter: fn(usize) -> String,
}

impl Histogram {
    pub fn new(encoder: fn(usize) -> usize, formatter: fn(usize) -> String) -> Self {
        Histogram {
            data: Vec::new(),
            data_sums: Vec::new(),
            encoder,
            formatter,
        }
    }

    pub fn add(&mut self, value: usize, sub_value: usize) {
        let index = (self.encoder)(value);
        self.grow(index + 1);
        self.data[index] += 1;
        self.data_sums[index] += sub_value;
    }

    pub fn to_string(&self, info: &str, show_avg: bool) -> String {
        let mut result = String::new();
        let (c_value, mut c_count, c_data, c_avg) = (15, 15, 50, 10);
        let mut count_lbl = "count";
        if show_avg {
            c_count += c_avg;
            count_lbl = "count / avg size";
        }
        let max = self.data.iter().max().unwrap();
        let per_item = max / c_data;
        writeln!(
            &mut result,
            "\n{info}. Each '∎' represents {} features.",
            per_item.separated_string()
        )
        .unwrap();
        writeln!(
            &mut result,
            "{:^c_value$} {:^c_count$}  {:^c_data$}",
            "value", count_lbl, "distribution"
        )
        .unwrap();
        writeln!(
            &mut result,
            "{:^c_value$} {:^c_count$}  {:^c_data$}",
            "-".repeat(c_value),
            "-".repeat(c_count),
            "-".repeat(c_data)
        )
        .unwrap();
        for index in 0..self.data.len() {
            print!("{:>c_value$} ", (self.formatter)(index));
            let count = self.data[index].separated_string();
            if show_avg {
                print!("{:>width$}", count, width = c_count - c_avg);
                let avg = self.data_sums[index] as f32 / self.data[index] as f32;
                print!(" /{:>width$.1} : ", avg, width = c_avg - 3);
            } else {
                print!("{:>c_count$}: ", count);
            }
            for _ in 0..(self.data[index] / per_item) {
                print!("∎");
            }
            writeln!(&mut result).unwrap();
        }
        result
    }

    fn grow(&mut self, length: usize) {
        if length > self.data.len() {
            for _ in 0..length - self.data.len() {
                self.data.push(0);
                self.data_sums.push(0);
            }
        }
    }
}

impl AddAssign for Histogram {
    fn add_assign(&mut self, other: Self) {
        self.grow(other.data.len());
        for i in 0..other.data.len() {
            self.data[i] += other.data[i];
            self.data_sums[i] += other.data_sums[i];
        }
    }
}

pub fn timed<F, R>(msg: &str, func: F) -> R
where
    F: FnOnce() -> R,
{
    let start = Instant::now();
    let res = func();
    println!("{msg} in {:.1} seconds", start.elapsed().as_secs_f32());
    res
}

pub fn spawn_stats_aggregator<T: 'static + Default + AddAssign + Debug + Send>(
    msg: &'static str,
    receiver: Receiver<T>,
) -> JoinHandle<()> {
    thread::spawn(move || {
        let start = Instant::now();
        let mut last_report = Instant::now();
        let mut stats = T::default();
        while let Ok(v) = receiver.recv() {
            stats += v;
            if last_report.elapsed().as_secs() > 60 {
                println!("{:.1}: {:?}", start.elapsed().as_secs_f32(), stats);
                last_report = Instant::now();
            }
        }
        println!("{} results: {:#?}", msg, stats);
    })
}

#[repr(i32)]
#[derive(Debug, ArgEnum, Clone, Copy)]
pub enum MemAdvice {
    Normal,
    Random,
    Sequential,
    WillNeed,
    DontNeed,
    #[cfg(target_os = "linux")]
    Free,
    #[cfg(target_os = "linux")]
    Remove,
    #[cfg(target_os = "linux")]
    DontFork,
    #[cfg(target_os = "linux")]
    DoFork,
    #[cfg(target_os = "linux")]
    Mergeable,
    #[cfg(target_os = "linux")]
    Unmergeable,
    #[cfg(target_os = "linux")]
    HugePage,
    #[cfg(target_os = "linux")]
    NoHugePage,
    #[cfg(target_os = "linux")]
    DontDump,
    #[cfg(target_os = "linux")]
    DoDump,
    #[cfg(target_os = "linux")]
    HwPoison,
}

#[cfg(unix)]
impl From<MemAdvice> for Advice {
    fn from(value: MemAdvice) -> Self {
        match value {
            MemAdvice::Normal => Advice::Normal,
            MemAdvice::Random => Advice::Random,
            MemAdvice::Sequential => Advice::Sequential,
            MemAdvice::WillNeed => Advice::WillNeed,
            MemAdvice::DontNeed => Advice::DontNeed,
            #[cfg(target_os = "linux")]
            MemAdvice::Free => Advice::Free,
            #[cfg(target_os = "linux")]
            MemAdvice::Remove => Advice::Remove,
            #[cfg(target_os = "linux")]
            MemAdvice::DontFork => Advice::DontFork,
            #[cfg(target_os = "linux")]
            MemAdvice::DoFork => Advice::DoFork,
            #[cfg(target_os = "linux")]
            MemAdvice::Mergeable => Advice::Mergeable,
            #[cfg(target_os = "linux")]
            MemAdvice::Unmergeable => Advice::Unmergeable,
            #[cfg(target_os = "linux")]
            MemAdvice::HugePage => Advice::HugePage,
            #[cfg(target_os = "linux")]
            MemAdvice::NoHugePage => Advice::NoHugePage,
            #[cfg(target_os = "linux")]
            MemAdvice::DontDump => Advice::DontDump,
            #[cfg(target_os = "linux")]
            MemAdvice::DoDump => Advice::DoDump,
            #[cfg(target_os = "linux")]
            MemAdvice::HwPoison => Advice::HwPoison,
        }
    }
}

#[derive(Debug, Args, Clone)]
pub struct OptAdvice {
    /// Let OS know how we plan to use the memmap
    #[cfg(unix)]
    #[clap(short, long, arg_enum)]
    pub advice: Vec<MemAdvice>,

    #[cfg(not(unix))]
    #[clap(skip = Vec::new())]
    advice: Vec<MemAdvice>,
}

pub fn advise_cache(cache: &DenseFileCache, advice: &OptAdvice) -> Result<(), Error> {
    #[cfg(unix)]
    for advice in &advice.advice {
        let adv = *advice;
        println!("Advising memmap as {adv:?}");
        cache
            .advise(Advice::try_from(adv)?)
            .with_context(|| format!("Unable set {adv:?}"))?;
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct NodeStats {
    pub node_count: usize,
    pub min_node_id: i64,
    pub max_node_id: i64,
    pub min_latitude: f64,
    pub max_latitude: f64,
    pub min_longitude: f64,
    pub max_longitude: f64,
}

impl NodeStats {
    pub fn add_node(&mut self, node_id: i64, lat: f64, lng: f64) {
        *self = Self {
            node_count: self.node_count + 1,
            min_node_id: self.min_node_id.min(node_id),
            max_node_id: self.max_node_id.max(node_id),
            min_latitude: self.min_latitude.min(lat),
            max_latitude: self.max_latitude.max(lat),
            min_longitude: self.min_longitude.min(lng),
            max_longitude: self.max_longitude.max(lng),
        };
    }
}

impl Default for NodeStats {
    fn default() -> Self {
        Self {
            node_count: 0,
            min_node_id: i64::MAX,
            max_node_id: i64::MIN,
            min_latitude: 0.0,
            max_latitude: 0.0,
            min_longitude: 0.0,
            max_longitude: 0.0,
        }
    }
}

impl AddAssign for NodeStats {
    fn add_assign(&mut self, other: Self) {
        *self = Self {
            node_count: self.node_count + other.node_count,
            min_node_id: self.min_node_id.min(other.min_node_id),
            max_node_id: self.max_node_id.max(other.max_node_id),
            min_latitude: self.min_latitude.min(other.min_latitude),
            max_latitude: self.max_latitude.max(other.max_latitude),
            min_longitude: self.min_longitude.min(other.min_longitude),
            max_longitude: self.max_longitude.max(other.max_longitude),
        };
    }
}
