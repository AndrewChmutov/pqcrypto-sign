mod common;
mod falcon;
mod fft;
mod encoding;
mod samplerz;
mod ntt;

struct Solution;

impl Solution {
    fn is_palindrome(s: &str) -> Option<&str> {
        let rev = s.chars().rev().collect::<String>();
        if s == rev {
            Some(s)
        } else {
            None
        }
    }

    pub fn longest_palindrome(s: String) -> String {
        Self::is_palindrome(&s).
    }
}

fn main() {
    println!("Hello world!");
}
