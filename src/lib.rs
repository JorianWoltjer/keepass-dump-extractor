use itertools::Itertools;
use std::collections::{HashMap, HashSet};

use cli::Format;

pub mod cli;

const PREFIX: [u8; 2] = [0xCF, 0x25]; // UTF-16-LE encoded '●' (U+25CF)
                                      // const VALID_CHARS: HashSet<char> =
lazy_static::lazy_static!(
    static ref VALID_CHARS: HashSet<char> = {
        let mut set = HashSet::new();
        for range in [0x20..=0x7E, 0x0A0..=0x2AF, 0x1E00..=0x1EFF] {
            set.extend(range.filter_map(char::from_u32));
        }
        set
    };
);

fn is_valid_char(c: char) -> bool {
    // let c = c as u32;
    // (0x20..=0x7E).contains(&c)
    //     // Latin characters (https://en.wikipedia.org/wiki/Latin_script_in_Unicode)
    //     || (0x00A0..=0x2AF).contains(&c)
    //     || (0x1E00..=0x1EFF).contains(&c)
    VALID_CHARS.contains(&c)
}

pub fn find_leaks(bytes: &[u8]) -> Vec<(usize, char)> {
    let mut results = Vec::new();
    let mut length = 0;
    let mut i = 0;
    while i < bytes.len() - 1 {
        if bytes[i..i + 2] == PREFIX {
            length += 1;
            i += 2;
            continue;
        } else if length > 0 {
            // Leaks are encoded in UTF-16-LE
            if let Some(Ok(c)) =
                char::decode_utf16([u16::from_le_bytes([bytes[i], bytes[i + 1]])]).next()
            {
                if is_valid_char(c) && bytes[i + 2..i + 4] == [0x00, 0x00] {
                    results.push((length, c));
                    i += 4;
                    continue;
                }
            }
        }
        length = 0;
        i += 1;
    }

    results
}

fn leak_to_string((length, c): (usize, char)) -> String {
    format!("{}{}", "●".repeat(length), c)
}

fn group_by_length(leaks: Vec<(usize, char)>) -> HashMap<usize, HashSet<(usize, char)>> {
    let mut map = HashMap::new();
    for leak in leaks {
        map.entry(leak.0).or_insert(HashSet::new()).insert(leak);
    }

    map
}

fn order_by_duplicates(leaks: &[(usize, char)]) -> Vec<(usize, char)> {
    let mut map = HashMap::new();
    for leak in leaks {
        map.entry(leak).or_insert(0);
        *map.get_mut(&leak).unwrap() += 1;
    }

    let mut leaks = map.into_iter().collect::<Vec<_>>();
    leaks.sort_by(|((a1, _), a2), ((b1, _), b2)| match a1.cmp(b1) {
        std::cmp::Ordering::Equal => a2.cmp(b2).reverse(),
        other => other,
    });
    leaks.into_iter().map(|(leak, _)| *leak).collect()
}

pub fn print_formatted_leaks(leaks: &[(usize, char)], format: cli::Format) {
    match format {
        Format::Found => {
            let leaks = order_by_duplicates(leaks);

            for leak in leaks {
                println!("{}", leak_to_string(leak));
            }
        }
        Format::Gaps => {
            todo!()
        }
        Format::All => {
            let leaks = group_by_length(leaks.to_vec());
            let max_length = *leaks.keys().max().unwrap() + 1;
            let unknowns = leaks.iter().filter(|(_, chars)| chars.len() > 1);
            let knowns = leaks
                .iter()
                .filter(|&(_, chars)| (chars.len() == 1))
                .map(|(_, chars)| chars.iter().next().unwrap());
            for perm in unknowns.map(|(_, set)| set).multi_cartesian_product() {
                let mut password = vec!['●'; max_length];
                for (length, c) in perm {
                    password[*length] = *c;
                }
                for (length, c) in knowns.clone() {
                    password[*length] = *c;
                }
                println!("{}", password.iter().collect::<String>());
            }
        }
    }
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

    #[test]
    fn format() {
        assert_eq!(leak_to_string((3, 'g')), "●●●g");
        assert_eq!(leak_to_string((1, 'g')), "●g");
    }
}
