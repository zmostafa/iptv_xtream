# IPTV XTREAM

IPTV Player written in Rust.
### ⚡ Notification

I am building this app for fun and also for my Rust learning journy, and I do not have the experience to design a nice UX, later also will be a huge refactoring as now I consider it a PoC project.

#### Building app with  `egui`

To build the `egui` version, you need to checkout branch `dev/v1.2` then run `cargo run --release` 

#### Building app with  `slint` 

To build the `egui` version, you need to checkout branch `dev/vSlint` then run `cargo run --release` 


The app comes with two UI falvors, one using `egui` crate and the other using `slint`

- [ ] Add new player embdded in gui `ffmpeg`

- [x] Fix fetching content

- [x] Fetch content only when required.

- [x] Add caching (Done Partially)

- [ ] Model gui on Series Troxide, but Arabic support using iced is not guranteed. 

- [x] Add downloading content

- [ ] use libmpv to integrate player in egui

- [ ] port gui to slint -- supports arabic

### ⚠️ Limitations
Ui in `egui` does not display arabic letters well, due to the fact its lack of RTL support.

### ⚠️ Warning

This application does not provide content or TV channels, it is a player application which streams from IPTV providers.