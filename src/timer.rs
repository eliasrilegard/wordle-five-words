use std::time::Instant;

pub struct Timer {
  start: Instant
}

impl Timer {
  pub fn new() -> Timer {
    Timer { start: Instant::now() }
  }

  pub fn elapsed_time(&self) -> String {
    let mut duration = self.start.elapsed().as_millis();
    let hours = duration / 3_600_000;

    duration = duration - hours * 3_600_000;
    let minutes = duration / 60_000;

    duration = duration - minutes * 60_000;
    let seconds = duration / 1_000;

    duration = duration - seconds * 1_000;
    format!("{:0>2}:{:0>2}:{:0>2}.{:0>3}", hours, minutes, seconds, duration)
  }
}