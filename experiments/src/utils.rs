use clap::ArgEnum;
use osmnodecache::Advice;
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
        let mut stats = T::default();
        while let Ok(v) = receiver.recv() {
            stats += v;
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
