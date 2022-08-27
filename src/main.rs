use std::{
  io::{
    BufRead,
    BufReader
  },
  fs::File
};

mod timer;
use timer::Timer;

fn load_words(path: &str) -> Vec<String> {
  let file = File::open(path).unwrap();
  BufReader::new(file).lines()
    .map(|l| l.unwrap())
    .collect::<Vec<String>>()
}

fn encode_word(word: &String) -> i32 {
  let mut bitset = 0;
  for c in word.chars() {
    let backshift = c as i32 - 96;
    bitset |= 1 << 26 >> backshift;
  }
  bitset
}

fn decode_words(words: Vec<i32>, raw: &Vec<String>) -> String {
  words.iter()
    .map(|word| decode_word(word, raw))
    .collect::<Vec<_>>()
    .join("\n")
}

fn decode_word(word: &i32, raw: &Vec<String>) -> String {
  let matches = raw.iter().filter(|raw| encode_word(raw) == *word).map(|word| word.clone()).collect::<Vec<_>>();
  format!("{} {}", visualize_word(word.clone()), matches.join("/"))
}

fn visualize_word(mut word: i32) -> String {
  let mut chars: [char; 26] = ['-'; 26];
  word <<= 6;

  for i in 0..chars.len() {
    if word < 0 {
      chars[i] = char::from_u32('A' as u32 + i as u32).unwrap();
    }
    word <<= 1;
  }

  chars.iter().collect()
}

fn main() {
  let timer = Timer::new();

  let raw_words = load_words("src/words.txt");
  let mut cooked_words = raw_words.iter().clone()
    .map(encode_word)
    .filter(|word| word.count_ones() == 5)
    .collect::<Vec<_>>();
  cooked_words.sort();
  cooked_words.dedup();

  let length = cooked_words.len();

  println!("{} raw words", raw_words.len());
  println!("{} cooked words", length);

  let mut count = 0;

  for i in 0..length {
    // println!("{}", i);
    let a = cooked_words[i];

    for j in (i + 1)..length {
      let b = cooked_words[j];
      if a & cooked_words[j] != 0 { continue; }
      let ab = a | b;

      for k in (j + 1)..length {
        let c = cooked_words[k];
        if ab & c != 0 { continue; }
        let abc = ab | c;

        for l in (k + 1)..length {
          let d = cooked_words[l];
          if abc & d != 0 { continue; }
          let abcd = abc | d;

          for m in (l + 1)..length {
            let e = cooked_words[m];
            if abcd & e != 0 { continue }
            count += 1;

            let decoded = decode_words(vec![a, b, c, d, e], &raw_words);
            println!(
              "[{time}] Solution {count}\n{words}\n",
              words = decoded,
              time = timer.elapsed_time(),
              count = count
            );
          }
        }
      }
    }
  }
  println!(
    "Completion time: {time}\n{count} solutions found.",
    time = timer.elapsed_time(),
    count = count
  );
}