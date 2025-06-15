//! Wordle solver main module.
//!
//! Provides logic for evaluating guesses, representing letter states, and loading word libraries.
//! Run this binary to test the guess evaluation logic.

// Standard library imports
use std::fs;
use std::path::Path;

/// State of a letter in a guess
#[derive(PartialEq)]
pub enum LetterState {

    /// The letter is in the correct position
    Correct,

    /// The letter is in the word, but not in this position, and the position it is in is not already marked as Correct
    Present,

    /// The letter is not in the word
    Absent,
}

/// Result of a guess
pub struct GuessResult {
    pub guess: String,
    states: Vec<LetterState>,
}

/// A library of valid words
pub struct Library {
    pub guesses: Vec<String>,
    pub answers: Vec<String>,
    pub word_length: usize,
}

/// Load words from a file into a vector of strings, ensuring all words have the same length.
/// Returns (words, word_length).
fn load_words_from_file(path: &Path) -> (Vec<String>, usize) {
    let contents: String = fs::read_to_string(path).expect("Something went wrong reading the file");
    let words: Vec<String> = contents.lines().map(|line| line.to_string()).collect();
    let word_length = words.first().map(|w| w.len()).unwrap_or(0);
    if !words.iter().all(|w| w.len() == word_length) {
        panic!("Not all words have the same length in file: {:?}", path);
    }
    (words, word_length)
}

impl Library {

    /// Load a library from a file
    pub fn load_from_file(guesses_path: &Path, answers_path: &Path) -> Library {
        let (guesses, guesses_word_length) = load_words_from_file(guesses_path);
        let (answers, answers_word_length) = load_words_from_file(answers_path);
        if guesses_word_length != answers_word_length {
            panic!("Guesses and answers must have the same word length: {} != {}", guesses_word_length, answers_word_length);
        }
        Library { guesses, answers, word_length: guesses_word_length }
    }

}

impl LetterState {

    /// Stringify letter state into emojis for console output
    pub fn to_string(&self) -> char {
        match self {
            LetterState::Correct => '🟩',
            LetterState::Present => '🟨',
            LetterState::Absent => '🟥',
        }
    }

}

impl GuessResult {

    /// Compares two strings of equal length and returns a Vec of LetterState
    pub fn evaluate_guess(guess: &str, answer: &str) -> GuessResult {
        if guess.len() != answer.len() {
            panic!("Guess and answer must be the same length");
        }
        let guess_chars: Vec<char> = guess.chars().collect();
        let answer_chars: Vec<char> = answer.chars().collect();
        let states: Vec<LetterState> = (0..guess_chars.len()).map(|i: usize| {
            evaluate_letter(&guess_chars, &answer_chars, i)
        }).collect();
        GuessResult { guess: guess.to_string(), states }
    }

    /// Stringify guess result into emojis for console output
    pub fn to_string(&self) -> String {
        self.states.iter().map(|s| s.to_string()).collect()
    }

}

/// Evaluates a single letter in a guess against the answer
fn evaluate_letter(guess: &[char], answer: &[char], i: usize) -> LetterState {
    let g: char = guess[i];
    let a: char = answer[i];
    if g == a {
        LetterState::Correct
    } else if answer.iter().enumerate().any(|(j, &ac)| j != i && ac == g && guess[j] != ac) {
        LetterState::Present
    } else {
        LetterState::Absent
    }
}

#[cfg(test)]
mod tests {

    // Standard library imports
    use std::path::PathBuf;
    use std::sync::OnceLock;

    // External crate imports
    use indicatif::ProgressBar;
    use indicatif::ProgressStyle;

    // Local crate imports
    use super::*;

    /// Load a library fixture for testing
    fn create_library_fixture() -> &'static Library {
        static LIBRARY: OnceLock<Library> = OnceLock::new();
        LIBRARY.get_or_init(|| {
            // Get paths to the data files
            let repo_root: &Path = Path::new(env!("CARGO_MANIFEST_DIR"));
            let data_root: PathBuf = repo_root.join("tests/data");
            let guesses_path: PathBuf = data_root.join("allowed.txt");
            let answers_path: PathBuf = data_root.join("allowed.txt");
            // Load the library from the files
            Library::load_from_file(&guesses_path, &answers_path)
        })
    }

    /// Create a progress bar template for displaying guess evaluation progress
    fn create_guess_progress_bar_template() -> &'static ProgressStyle {
        static PROGRESS_STYLE: OnceLock<ProgressStyle> = OnceLock::new();
        PROGRESS_STYLE.get_or_init(|| {
            let library = create_library_fixture();
            let bar_template: String = format!(
                concat!(
                    "{{elapsed_precise}} / {{duration_precise}} | ",
                    "{{msg:{msg_width}}} | ",
                    "{{pos:>{len_width}}}/{{len:{len_width}}} | ",
                    "{{wide_bar}}",
                ),
                len_width = (library.guesses.len() as f64).log10().ceil() as usize,
                msg_width = library.word_length
            );
            ProgressStyle::with_template(&bar_template).unwrap()
        })
    }

    #[test]
    #[ignore = "This test is slow and should not run by default"]
    fn test_evaluate_all_guess_answer_pairs() {

        // Load the library fixture
        let library: &Library = create_library_fixture();

        // Create a progress bar for the guesses
        let bar: ProgressBar = ProgressBar::new(library.guesses.len() as u64);
        let bar_style: &ProgressStyle = create_guess_progress_bar_template();
        bar.set_style(bar_style.clone());

        // Iterate over the guesses and evaluate each one against all answers
        for guess in bar.wrap_iter(library.guesses.iter()) {
            bar.set_message(guess);
            let guess_chars: Vec<char> = guess.chars().collect();
            for answer in &library.answers {
                let answer_chars: Vec<char> = answer.chars().collect();

                // Check result
                let result: GuessResult = GuessResult::evaluate_guess(guess, answer);
                
                // Double check results
                for (index, state) in result.states.iter().enumerate() {
                    let guess_char = guess_chars[index];
                    let answer_char = answer_chars[index];
                    match state {
                        LetterState::Correct => {
                            // Ensure the guess letter matches the answer letter at this index
                            assert_eq!(
                               guess_char, answer_char,
                                "Expected correct letter at index {}: {} != {}", index, guess_char, answer_char
                            );
                        },
                        LetterState::Absent => {
                            // Ensure the guess letter does not match the answer letter at this index
                            assert_ne!(
                                guess_char, answer_char, 
                                "Expected letter {} to be present in answer {} but not in a correct position",
                                guess_char, answer
                            );
                            // Ensure the guess letter is not present in the answer (unless it is at a correct position)
                            assert!(
                                !answer_chars.iter().enumerate().any(|(j, &ac)| {
                                    ac == guess_char && result.states[j] != LetterState::Correct
                                }),
                                "Expected letter {} to be absent in answer {}", guess_char, answer
                            );
                        },
                        LetterState::Present => {
                            // Ensure the guess letter does not match the answer letter at this index
                            assert_ne!(
                                guess_char, answer_char, 
                                "Expected letter {} to be absent in answer {}", guess_char, answer
                            );
                            // Ensure the guess letter is present in the answer somewhere that is not a correct position
                            assert!(
                                answer_chars.iter().enumerate().any(|(j, &ac)| {
                                    ac == guess_char && result.states[j] != LetterState::Correct
                                }),
                                "Expected letter {} to be absent in answer {}", guess_char, answer
                            );
                        },

                    }
                }
            }
        }
        bar.finish_with_message("All guesses evaluated");

    }

}