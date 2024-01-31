pub mod cli;

const PREFIX: [u8; 2] = [0xCF, 0x25]; // UTF-16-LE encoded '●' (U+25CF)

fn is_valid_char(c: char) -> bool {
    let b = c as u32;
    c.is_ascii_graphic()
        // Latin characters (https://en.wikipedia.org/wiki/Latin_script_in_Unicode)
        || (0x00A0..=0x2AF).contains(&b)
        || (0x1E00..=0x1EFF).contains(&b)
}

pub fn find_leaks(bytes: &[u8]) -> Vec<(usize, char)> {
    let mut results = Vec::new();
    let mut count = 0;
    let mut found = None;
    // Leaks are aligned to 2 bytes, allows for easier searching
    for ab in bytes.chunks(2) {
        if ab == PREFIX {
            count += 1;
            continue;
        } else if count > 0 {
            if let Some(result) = found {
                // Leaks must end with null bytes
                if ab == [0x00, 0x00] {
                    results.push(result);
                    found = None;
                }
            }
            // Leaks are encoded in UTF-16-LE
            if let Some(Ok(c)) = char::decode_utf16([u16::from_le_bytes([ab[0], ab[1]])]).next() {
                if is_valid_char(c) {
                    found = Some((count, c));
                    continue;
                }
            }
        }
        count = 0;
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;

    #[test]
    fn simple_character() {
        let input = hex!("cf2500004141414141414141cf25cf25cf25670000004242424242424242");

        assert_eq!(find_leaks(&input), vec![(3, 'g')]);

        let input = hex!("cf25cf25cf25cf25cf25cf25cf25cf25cf25cf25cf25cf2541000000");
        assert_eq!(find_leaks(&input), vec![(12, 'A')]);
    }

    #[test]
    fn non_ascii_character() {
        // Uses UTF-16-LE encoding
        let input = hex!("cf25cf25cf25cf2553010000");

        assert_eq!(find_leaks(&input), vec![(4, 'œ')]);
    }
}
