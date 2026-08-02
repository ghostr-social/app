# Manual verification

## Android — 2026-08-02

Environment: disposable Android 36.1 x86_64 emulator, using the debug APK.

Verified:

- The sign-in screen rendered and treated the nsec field as a password.
- A deterministic test nsec opened the authenticated application shell.
- Reinstalling the APK restored the authenticated session from secure storage.
- Feed, search, compose, activity, and profile navigation remained available.
- Compose exposed both gallery and camera actions while Publish stayed disabled
  without a selected draft.
- Denying camera access returned the app-safe permission message and kept
  Publish disabled.
- Granting camera access opened Android's camera in video mode; the capture was
  cancelled without recording media.
- Choosing the gallery opened Android's system photo picker; the selection was
  cancelled without granting the app access to user media.
- The current-user profile showed the derived Nostr identity, sign-out action,
  and settings entry.
- Settings showed editable relay and Blossom endpoints and the selected 2 GB
  video inventory budget.
- The packaged Rust gateway started without the prior unavailable-gateway
  state. The feed showed its normal loading state, and logs contained no Dart,
  Flutter, or Rust panic/exception.

The disposable emulator had no external network access, so live relay feed
population, remote video playback, Blossom upload, and publication were not
manually exercised. Automated adapter, widget, cache, fallback, upload,
publication, and native gateway tests cover those boundaries.

After the typed Rust startup bridge was regenerated, packaging was reverified:

- The debug APK is x86_64-only. Its four native libraries are
  `libdatastore_shared_counter.so`, `libflutter.so`,
  `librust_lib_ghostr.so`, and `librust_lib_ndk.so` under `lib/x86_64/`.
  The APK is 121,126,661 bytes, its packaged `librust_lib_ghostr.so` is
  20,821,392 bytes, and its SHA-256 is
  `d73bc6fb1916973ae7d0b6ee0a3b71ea53f98bf2b4cf5095a16801a7cf6d9bb6`.
- The release APK is arm64-v8a-only. Its five native libraries are `libapp.so`,
  `libdatastore_shared_counter.so`, `libflutter.so`,
  `librust_lib_ghostr.so`, and `librust_lib_ndk.so` under `lib/arm64-v8a/`.
  The APK is 30,510,286 bytes, its packaged `librust_lib_ghostr.so` is
  8,567,064 bytes, and its SHA-256 is
  `6db952cdc5f38602666b5c26e8df60ae8f73e14def6a8261a34ee82d9b97932f`.

The disposable AVD was deleted after verification. No unrelated applications
or user data were removed.
