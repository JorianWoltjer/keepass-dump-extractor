use itertools::Itertools;
use once_cell::sync::Lazy;
use std::collections::{HashMap, HashSet};

use cli::Format;

pub mod cli;

const PREFIX: [u8; 2] = [0xCF, 0x25]; // UTF-16-LE encoded '●' (U+25CF)
                                      // const VALID_CHARS: HashSet<char> =
static VALID_CHARS: Lazy<HashSet<char>> = Lazy::new(|| {
    let mut set = HashSet::new();
    for range in [0x20..=0x7E, 0x0A0..=0x2AF, 0x1E00..=0x1EFF] {
        set.extend(range.filter_map(char::from_u32));
    }
    set
});
static COMMON_CHARS: Lazy<HashSet<char>> =
    Lazy::new(|| HashSet::from_iter((0x20..=0x7E).filter_map(char::from_u32)));

pub fn find_leaks(bytes: &[u8]) -> Vec<(usize, char)> {
    let mut results = Vec::new();
    let mut length = 0;
    let mut i = 0;
    while i < bytes.len() - 1 {
        // Leaks must begin with a series of '●'
        if bytes[i..i + 2] == PREFIX {
            length += 1;
            i += 2;
            continue;
        } else if length > 0 {
            // Leaks are encoded in UTF-16-LE
            if let Some(Ok(c)) =
                char::decode_utf16([u16::from_le_bytes([bytes[i], bytes[i + 1]])]).next()
            {
                // Filter some uncommon characters, and check for null bytes
                if VALID_CHARS.contains(&c) && bytes[i + 2..i + 4] == [0x00, 0x00] {
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

fn get_unknowns_and_knowns(leaks: Vec<(usize, char)>) -> (Vec<HashSet<(usize, char)>>, Vec<char>) {
    let leaks = group_by_length(leaks);
    let max_length = *leaks.keys().max().unwrap() + 1;

    let unknowns = (0..max_length)
        .filter_map(|length| {
            let chars = leaks.get(&length).cloned().unwrap_or_else(|| {
                // Insert all common characters if there are no leaks of this length
                HashSet::from_iter((*COMMON_CHARS).iter().map(|c| (length, *c)))
            });
            (chars.len() > 1).then_some(chars)
        })
        .collect::<Vec<_>>();
    let mut password = vec!['●'; max_length];
    leaks.iter().for_each(|(length, chars)| {
        if chars.len() == 1 {
            password[*length] = chars.iter().next().unwrap().1;
        }
    });
    (unknowns, password)
}

pub fn print_formatted_leaks(leaks: &[(usize, char)], format: cli::Format) {
    match format {
        // Directly print all hints about the password
        Format::Found => {
            let leaks = order_by_duplicates(leaks);

            for leak in leaks {
                println!("{}", leak_to_string(leak));
            }
        }
        // Summarize the hints into the full size, leaving gaps for unknown characters
        Format::Gaps => {
            let (unknowns, password) = get_unknowns_and_knowns(leaks.to_vec());

            for unknown in unknowns {
                // TODO: maybe sort here
                for (length, c) in unknown {
                    let mut password = password.clone();
                    password[length] = c;
                    println!("{}", password.iter().collect::<String>());
                }
            }
        }
        // Print all possible permutations of the password
        Format::All => {
            let (unknowns, mut password) = get_unknowns_and_knowns(leaks.to_vec());

            for perm in unknowns.iter().multi_cartesian_product() {
                for (length, c) in perm {
                    // No need to clone because next iteration will overwrite everything
                    password[*length] = *c;
                }
                println!("{}", password.iter().collect::<String>());
            }
        }
        // Write the raw results with all found information, not intended for human consumption
        Format::Raw => {
            todo!("Write count, length and char");
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
