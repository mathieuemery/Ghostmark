# ghostmark

Visually invisible per-recipient watermarking using Unicode homoglyphs. Helps you find out who leaked your document.

## Why this is useful

If you send the same sensitive text to several people and it leaks, you normally have no way to know which copy it came from.

This tool makes every copy *look* identical but actually be slightly different. For each recipient, a handful of letters in the text are swapped for visually-identical characters from another script (for example, a Cyrillic "а" instead of a Latin "a"). To the human eye the text is unchanged, but the exact combination of swapped letters is unique to that recipient.

If a leaked copy turns up later, you compare it against your original text and your recipient list, and the tool tells you which recipient's copy it most closely matches.

Unlike zero-width character tricks, homoglyph swaps are ordinary Unicode characters and often survive copy-paste and forwarding. However, Unicode normalization, sanitization, OCR, and some document conversions can alter or remove them.

## How to use it

### 1. Generate a secret key

```bash
╰─λ openssl rand -out secret.key 32
```

### 2. Prepare your files

- A cover text file: the message you're sending (`cover.txt`)
- A recipients CSV: one email per line (`recipients.csv`)

### 3. Encode: generate one watermarked copy per recipient

```bash
╰─λ ./ghostmark encode --cover cover.txt --recipients recipients.csv --key-file secret.key --out-dir ./out
```

This writes one `.txt` file per recipient into `./out`, each with a unique combination of homoglyph swaps. Send each recipient their specific file.

### 4. Identify: find out who leaked a copy

If a copy leaks and you get hold of the leaked text, save it to a file and run:

```bash
 ╰─λ ./ghostmark identify --cover cover.txt --leaked leaked.txt --recipients recipients.csv --key-file secret.key
```

This prints every candidate ranked by how well their expected watermark matches the leaked text, with the best match at the top:

```
Candidate                                Matches       Rate
------------------------------------------------------------
bob@example.com                           59/59      100.0%
alice@example.com                         36/59       61.0%
erin@example.com                          35/59       59.3%
karl@example.com                          34/59       57.6%
jade@example.com                          33/59       55.9%
... and 5 more (use --all to show everyone)

Most likely source: bob@example.com (59 matches out of 59)
```

A clear winner with close to 100% and a big gap to the runner-up means high confidence. A near-tie means the leaked text may have been edited, or your cover text didn't have enough eligible letters to distinguish recipients reliably.

## Why there's a private key

Without a secret key, anyone who understood the scheme could compute what a given recipient's watermark *should* look like, and fabricate a fake "leaked" copy to frame someone, or strip/repro­duce watermarks freely.

With the secret key, each recipient's watermark pattern is derived using [HKDF](https://en.wikipedia.org/wiki/HKDF) from their email address as info *and* your secret key as IKM. Without the key, it's computationally infeasible to predict or reproduce any recipient's exact pattern, even if someone knows exactly how the tool works, has the cover text, and has the full recipient list.

This means:

- **A match is difficult to forge.** A valid match is difficult to fabricate without the secret key. However, it does not prove who physically leaked the document; it identifies which recipient-specific copy the leaked text most closely matches
- **The key must stay secret.** Anyone with the key can forge or verify any recipient's watermark. Keep `secret.key` off version control, off shared drives, and out of any client-side code. If it ever leaks, treat all watermarks generated with that key as compromised and generate a new key for future batches.
- **Reuse the same key across a whole encode/identify cycle**, but you can rotate it for a new batch of documents whenever you like. There's no need to keep using the same key forever.

## Examples

Here's what two watermarked copies of the same sentence look like side by side:

**Copy sent to alice@example.com:**
```
Hi thеre, pleаsе fіnd attасhed the confіdеntial Q3 rоаdmap dосument fоr уоur rеvіew.
```

**Copy sent to bob@example.com:**
```
Hi there, pleаѕе fіnd аttасhed thе cоnfidеntiаl Q3 rоаdmap doсument for уоur reviеw.
```

A handful of letters in each copy have quietly been swapped for lookalike characters from a different Unicode script.

## A note on what this is (and isn't)

This is a watermarking / fingerprinting tool, not a cryptographic signature scheme. It gives you strong, practical confidence about which recipient a leak came from, as long as:

- your secret key never left your control, and
- the leaked text is a reasonably faithful copy-paste/forward of what was sent (not retyped from a photo, heavily paraphrased, or OCR'd from a screenshot).

It won't survive someone reading the text and summarizing it in their own words. No text watermark can.