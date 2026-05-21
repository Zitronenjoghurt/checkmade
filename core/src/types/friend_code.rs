pub const FRIEND_CODE_ALPHABET: &[char] = &[
    '2', '3', '4', '6', '7', '8', '9', 'B', 'D', 'F', 'G', 'H', 'K', 'M', 'P', 'R', 'T', 'X',
];
pub const FRIEND_CODE_LENGTH: usize = 9;

#[cfg(feature = "nanoid")]
pub fn generate_friend_code() -> String {
    nanoid::nanoid!(FRIEND_CODE_LENGTH, FRIEND_CODE_ALPHABET)
}

pub fn is_valid_friend_code(code: &str) -> bool {
    code.len() == FRIEND_CODE_LENGTH && code.chars().all(|c| FRIEND_CODE_ALPHABET.contains(&c))
}

pub fn friend_code_char_filter(c: char) -> Option<char> {
    let c = c.to_ascii_uppercase();
    FRIEND_CODE_ALPHABET.contains(&c).then_some(c)
}
