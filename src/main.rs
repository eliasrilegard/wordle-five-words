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

  /*
   * Generate a table storing the index of the first word that doesn't share any
   * letters with a given word with index A, starting at any starting point B.
   * Basically: The first word at or after B that doesn't collide with A.
   */
  let mut skip: Vec<Vec<u16>> = vec![vec![0; length + 1]; length];
  for i in 0..length {
    skip[i][length] = length as u16; // 5182
    let a = cooked_words[i];
    for j in (i..length).rev() {
      let b = cooked_words[j];
      skip[i][j] = if a & b == 0 { j as u16 } else { skip[i][j + 1] }
    }
  }
  /*
   * Practically the same as doing skip[x][y] on j,k,l,m assignments,
   * but storing these values in a smaller array is easier on the CPU cache
   */
  let mut first = vec![0; length];
  for i in 0..length {
    first[i] = skip[i][i];
  }

  let mut count = 0;

  for i in 0..length {
    // println!("{}", i);
    let a = cooked_words[i];

    let mut j = first[i] as usize;
    while j < length {
      let b = cooked_words[j];
      let ab = a | b;
      
      let mut k = first[j] as usize;
      while k < length {
        let c = cooked_words[k];
        if ab & c != 0 {
          k = skip[i][k + 1] as usize;
          k = skip[j][k] as usize;
          continue;
        }
        let abc = ab | c;
        
        let mut l = first[k] as usize;
        while l < length {
          let d = cooked_words[l];
          if abc & d != 0 {
            l = skip[i][l + 1] as usize;
            l = skip[j][l] as usize;
            l = skip[k][l] as usize;
            continue;
          }
          let abcd = abc | d;
          
          let mut m = first[l] as usize;
          while m < length {
            let e = cooked_words[m];
            if abcd & e != 0 {
              m = skip[i][m + 1] as usize;
              m = skip[j][m] as usize;
              m = skip[k][m] as usize;
              m = skip[l][m] as usize;
              continue;
            }
            count += 1;

            let decoded = decode_words(vec![a, b, c, d, e], &raw_words);
            println!(
              "[{time}] Solution {count}\n{words}\n",
              words = decoded,
              time = timer.elapsed_time(),
              count = count
            );

            m = skip[i][m + 1] as usize; // Go to the next word, find word that doesn't collide with A
            m = skip[j][m] as usize; // Then find the word that doesn't collide with B
            m = skip[k][m] as usize; // -- // -- C
            m = skip[l][m] as usize; // -- // -- D
          }
          l = skip[i][l + 1] as usize;
          l = skip[j][l] as usize;
          l = skip[k][l] as usize;
        }
        k = skip[i][k + 1] as usize;
        k = skip[j][k] as usize;
      }
      j = skip[i][j + 1] as usize;
    }
  }
  println!(
    "Completion time: {time}\n{count} solutions found.",
    time = timer.elapsed_time(),
    count = count
  );
}