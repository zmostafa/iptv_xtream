# IPTV XTREAM

IPTV Player with support for xtream APIs written in Rust.
### ⚡ Notification

I am building this app for fun and also for my Rust learning journy, and I do not have the experience to design a nice UX, later also will be a huge refactoring as now I consider it a PoC project.

#### Building app with  `egui`

To build the `egui` version, you need to checkout branch `dev/v1.2` then run `cargo run --release` 

#### Building app with  `slint` 

To build the `egui` version, you need to checkout branch `dev/vSlint` then run `cargo run --release` 

##### Dependancies

The app uses `mpv` player for playing the streams, also uses `wget` for downloading the content for offline playing.
Make sure you install both applications before using the IPTV Player.


The app comes with two UI falvors, one using `egui` crate and the other using `slint`

- [ ] Add new player embdded in gui `ffmpeg` or `libmpv`

- [x] Fix fetching content
  - [ ] Implement in slint ui

- [x] Fetch content only when required.
  - [ ] Fix fetch serie issue.

- [x] Add caching (Done Partially)
  - [ ] Still needs working on making the switch to big categories faster

- [ ] Model gui on Series Troxide, but Arabic support using iced is not guranteed. 

- [x] Add downloading content
  - [ ] Implement in slint ui

- [ ] use libmpv to integrate player in egui

- [ ] port gui to slint -- supports arabic

- [ ] Add search and favorites views in slint
- [ ] Get casing information from TMDB website for better display
- [ ] Display Movie/Serie plot and info 
  - [ ] implement a new API to get the movie/serie info data.

### ⚠️ Limitations
Ui in `egui` does not display arabic letters well, due to the fact its lack of RTL support.

### ⚠️ Warning

This application does not provide content or TV channels, it is a player application which streams from IPTV providers.