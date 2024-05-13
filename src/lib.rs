mod data;
mod card;
mod arrange;
mod config;

pub mod app;

#[macro_export]
macro_rules! write_info {
    ($content:expr) => {{
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open("log.txt")
            .expect("Failed to open file");

        writeln!(file, "{}", $content).expect("Failed to write to file");
    }};
}
