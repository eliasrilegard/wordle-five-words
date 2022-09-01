use std::{
  io::{
    BufRead,
    BufReader
  },
  fs::File,
  sync::{
    Arc,
    Mutex
  },
  thread
};

mod timer;
use timer::Timer;

#[derive(Clone)]
struct Letter {
  name: char,
  occurences: u16
}

#[derive(Clone)]
struct Word {
  name: String,
  bitset: u32
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

fn encode_word(word: &String, weights: &Vec<u32>) -> Word {
  let mut bitset = 0;
  for c in word.chars() {
    bitset |= weights[alphabet_index(c)];
  }
  Word { name: word.to_string(), bitset }
}

fn decode_words(indices: Vec<u32>, words: &Vec<Word>, letters: &Vec<Letter>) -> String {
  indices.iter()
    .map(|&index| decode_index(index, words, letters))
    .collect::<Vec<_>>()
    .join("\n")
}

fn decode_index(index: u32, words: &Vec<Word>, letters: &Vec<Letter>) -> String {
  let i = index as usize;
  let bitset = words[i].bitset;
  let mut name = words[i].name.clone();
  let mut j = i + 1;
  while j < words.len() && words[j].bitset == bitset {
    name.push_str(&(String::from("/") + &words[j].name));
    j += 1;
  }
  format!("{} {}", visualize_word(bitset, letters), name)
}

fn visualize_word(bitset: u32, letters: &Vec<Letter>) -> String {
  let mut chars: [char; 26] = ['-'; 26];
  let mut word = (bitset << 6) as i32;
  let offset = 'a' as u32 - 'A' as u32;

  for letter in letters {
    let character = letter.name;
    if word < 0 {
      chars[alphabet_index(character)] = char::from_u32(character as u32 - offset).unwrap();
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

  // Array of all words, sorted such that anagrams are adjacent
  let mut words = raw_words.iter().clone()
    .map(|word| encode_word(word, &letter_weights))
    .filter(|word| {
      word.name.len() == 5 && word.bitset.count_ones() == 5
    })
    .collect::<Vec<_>>();
  words.sort_by(|a, b| b.bitset.cmp(&a.bitset));
  
  let mut cooked_words = words.iter().map(|w| w.bitset).collect::<Vec<_>>();
  cooked_words.dedup();
  
  /*
   * Find index of the first word that doesn't have any of the two rarest letters.
   * The variable name here is a reference to what two letters are the rarest,
   * but the code figures this out on the fly.
   */
  let xq = 1 << 25 | 1 << 24;
  let split = cooked_words.iter().position(|w| w & xq == 0).unwrap();

  /*
   * Array of indices for which all the words at and after are anagrams.
   * Each index is the corresponding cooked word's index in words.
   * Makes looking the word back up when a solution is found much faster.
   */
  let indices = cooked_words.iter()
    .map(|&bitset| {
      words.iter().position(|w| w.bitset == bitset).unwrap() as u32
    }).collect::<Vec<_>>();
  let length = cooked_words.len();

  println!("{} raw words", raw_words.len());
  println!("{} with 5 unique letters", words.len());
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

  let counter = Arc::new(Mutex::new(0));

  let cpu_count = thread::available_parallelism().unwrap().get();
  let mut handles: Vec<thread::JoinHandle<()>> = vec![];

  for t in 0..cpu_count {
    let range = (t..split).step_by(cpu_count);
    let counter_t = Arc::clone(&counter);
    
    let cooked_words_t = cooked_words.clone();
    let indices_t = indices.clone();
    let first_t = first.clone();
    let skip_t = skip.clone();
    let timer_t = timer.clone();
    let letters_t = letters.clone();
    let words_t = words.clone();
    
    let handle = thread::spawn(move || {
      for i in range {
        // println!("{}", i);
        let a = cooked_words_t[i];
        let i_chunk = i * (length + 1);
        
        let mut j = first_t[i] as usize;
        while j < length {
          let b = cooked_words_t[j];
          let ab = a | b;
          let j_chunk = j * (length + 1);
          
          let mut k = first_t[j] as usize;
          while k < length {
            let c = cooked_words_t[k];
            if ab & c != 0 {
              k = skip_t[i_chunk + k + 1] as usize;
              k = skip_t[j_chunk + k] as usize;
              continue;
            }
            let abc = ab | c;
            let k_chunk = k * (length + 1);
            
            let mut l = first_t[k] as usize;
            while l < length {
              let d = cooked_words_t[l];
              if abc & d != 0 {
                l = skip_t[i_chunk + l + 1] as usize;
                l = skip_t[j_chunk + l] as usize;
                l = skip_t[k_chunk + l] as usize;
                continue;
              }
              let abcd = abc | d;
              let l_chunk = l * (length + 1);
              
              let mut m = first_t[l] as usize;
              while m < length {
                let e = cooked_words_t[m];
                if abcd & e != 0 {
                  m = skip_t[i_chunk + m + 1] as usize;
                  m = skip_t[j_chunk + m] as usize;
                  m = skip_t[k_chunk + m] as usize;
                  m = skip_t[l_chunk + m] as usize;
                  continue;
                }
                let mut count = counter_t.lock().unwrap();
                *count += 1;

                let decoded = decode_words(vec![indices_t[i], indices_t[j], indices_t[k], indices_t[l], indices_t[m]], &words_t, &letters_t);
                println!(
                  "[{time}] Solution {count}\n{words}\n",
                  words = decoded,
                  time = timer_t.elapsed_time(),
                  count = count
                );

                m = skip_t[i_chunk + m + 1] as usize; // Go to the next word, find word that doesn't collide with A
                m = skip_t[j_chunk + m] as usize; // Then find the word that doesn't collide with B
                m = skip_t[k_chunk + m] as usize; // -- // -- C
                m = skip_t[l_chunk + m] as usize; // -- // -- D
              }
              l = skip_t[i_chunk + l + 1] as usize;
              l = skip_t[j_chunk + l] as usize;
              l = skip_t[k_chunk + l] as usize;
            }
            k = skip_t[i_chunk + k + 1] as usize;
            k = skip_t[j_chunk + k] as usize;
          }
          j = skip_t[i_chunk + j + 1] as usize;
        }
      }
    });

    handles.push(handle);
  }
  for h in handles {
    h.join().unwrap();
  }
  println!(
    "Completion time: {time}\n{count} solutions found.",
    time = timer.elapsed_time(),
    count = *counter.lock().unwrap()
  );
}