- **A copy can say whether it worked.** `Tx::copy_checked(text, f)` and
  `Tx::copy_rich_checked(content, f)` copy as `Tx::copy` and `Tx::copy_rich` do, then run `f`
  as an update with `Ok(())` once the clipboard holds it or `Err(reason)` when it doesn't (no
  clipboard, the browser refused), so an app can say "Couldn't copy" instead of claiming it
  copied. A copy that replaces it before it's written drops `f`.
