use std::{
  io::{
    BufRead,
    BufReader
  },
  fs::File
};

mod timer;
use timer::Timer;

struct Letter {
  name: char,
  occurences: u16
}

fn alphabet_index(letter: char) -> usize {
  letter as usize - 'a' as usize
}

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

  let mut letters: Vec<Letter> = vec![];
  for c in 'a'..='z' {
    letters.push(Letter { name: c, occurences: 0 });
  }
  for c in raw_words.join("").chars() {
    if c == '\n' { continue; }
    letters[alphabet_index(c)].occurences += 1;
  }
  letters.sort_by(|a, b| b.occurences.cmp(&a.occurences));
  letters.rotate_right(2);

  // Map every letter to a weight with respect to how common the letter is
  let mut letter_weights = vec![0; 26];
  for i in 0..26 {
    letter_weights[alphabet_index(letters[i].name)] = 1 << 25 >> i;
  }

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
   * This is now just a 1D array for performance reasons, but the functionality
   * and purpose is still identical.
   * Access to this "2D array" is always done in a [i * X + j] fashion,
   * treat i and j as the two indices.
   */
  let mut skip: Vec<u16> = vec![0; length * (length + 1)];
  /*
   * Practically the same as doing skip[x][x] on j,k,l,m assignments,
   * but storing these values in a smaller array is easier on the CPU cache
   */
  let mut first = vec![0; length];

  for i in 0..length {
    let mut next = length as u16; // 5182
    skip[i * (length + 1) + length] = next;
    let a = cooked_words[i];
    for j in (i..length).rev() {
      let b = cooked_words[j];
      if a & b == 0 {
        next = j as u16
      }
      skip[i * (length + 1) + j] = next;
    }
    first[i] = skip[i * (length + 1) + i];
  }

  let mut count = 0;

  for i in 0..length {
    // println!("{}", i);
    let a = cooked_words[i];
    let i_chunk = i * (length + 1);

    let mut j = first[i] as usize;
    while j < length {
      let b = cooked_words[j];
      let ab = a | b;
      let j_chunk = j * (length + 1);
      
      let mut k = first[j] as usize;
      while k < length {
        let c = cooked_words[k];
        if ab & c != 0 {
          k = skip[i_chunk + k + 1] as usize;
          k = skip[j_chunk + k] as usize;
          continue;
        }
        let abc = ab | c;
        let k_chunk = k * (length + 1);
        
        let mut l = first[k] as usize;
        while l < length {
          let d = cooked_words[l];
          if abc & d != 0 {
            l = skip[i_chunk + l + 1] as usize;
            l = skip[j_chunk + l] as usize;
            l = skip[k_chunk + l] as usize;
            continue;
          }
          let abcd = abc | d;
          let l_chunk = l * (length + 1);
          
          let mut m = first[l] as usize;
          while m < length {
            let e = cooked_words[m];
            if abcd & e != 0 {
              m = skip[i_chunk + m + 1] as usize;
              m = skip[j_chunk + m] as usize;
              m = skip[k_chunk + m] as usize;
              m = skip[l_chunk + m] as usize;
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

            m = skip[i_chunk + m + 1] as usize; // Go to the next word, find word that doesn't collide with A
            m = skip[j_chunk + m] as usize; // Then find the word that doesn't collide with B
            m = skip[k_chunk + m] as usize; // -- // -- C
            m = skip[l_chunk + m] as usize; // -- // -- D
          }
          l = skip[i_chunk + l + 1] as usize;
          l = skip[j_chunk + l] as usize;
          l = skip[k_chunk + l] as usize;
        }
        k = skip[i_chunk + k + 1] as usize;
        k = skip[j_chunk + k] as usize;
      }
      j = skip[i_chunk + j + 1] as usize;
    }
  }
  println!(
    "Completion time: {time}\n{count} solutions found.",
    time = timer.elapsed_time(),
    count = count
  );
}