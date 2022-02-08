use std::ops::AddAssign;
use separator::Separatable;

#[derive(Debug)]
pub struct Histogram {
    data: Vec<usize>,
    encoder: fn(usize) -> usize,
    formatter: fn(usize) -> String,
}

impl Histogram {
    pub fn new(encoder: fn(usize) -> usize, formatter: fn(usize) -> String) -> Self {
        Histogram { data: Vec::new(), encoder, formatter }
    }

    pub fn add(&mut self, value: usize) {
        let index = (self.encoder)(value);
        self.grow(index + 1);
        self.data[index] += 1;
    }

    pub fn print(&self, info: &str) {
        let (c_value, c_count, c_data) = (15, 15, 50);
        let max = self.data.iter().max().unwrap();
        let per_item = max / c_data;
        println!("\n{info}. Each '∎' represents {} features.", per_item.separated_string());
        println!("{:^c_value$} {:^c_count$}  {:^c_data$}", "value", "count", "distribution");
        println!("{:^c_value$} {:^c_count$}  {:^c_data$}", "-".repeat(c_value), "-".repeat(c_count), "-".repeat(c_data));
        for index in 0..self.data.len() {
            print!("{:>c_value$} {:>c_count$}: ", (self.formatter)(index), self.data[index].separated_string());
            for _ in 0..(self.data[index] / per_item) {
                print!("∎");
            }
            println!();
        }
    }
    fn grow(&mut self, length: usize) {
        if length > self.data.len() {
            for _ in 0..length - self.data.len() {
                self.data.push(0);
            }
        }
    }
}

impl AddAssign for Histogram {
    fn add_assign(&mut self, other: Self) {
        self.grow(other.data.len());
        for i in 0..other.data.len() {
            self.data[i] += other.data[i]
        }
    }
}
