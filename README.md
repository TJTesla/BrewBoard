# BrewBoard

The BrewBoard is a helper for brewing pour over coffee.
After being into this hobby for a few years, one aquires multiple brewers, each requiring multiple different recipes to get the best out of those (maybe) way to expensive coffee beans.
This project develops the software side for a system that runs on a Raspberry Pi (Model 3B in my case).
Connected to a touch display, it helps brewing coffee by dictating how the next pour is supposed to be done.
Additionally, the Raspberry Pi can connect to Bluetooth scales (since I only have the Bookoo Themis Ultra I have only implemented it for this one) and display the current weight, which allows the automatic switch two the showing the next pour

The recipes that can be followed can be send to the Raspberry Pi using a website which it hosts itself.
This website has three main functions:
1. Add new recipes (with an arbitrary number of pours)
2. Manage the existing recipes by listing them, editing them and deleting them
3. Manage brews that were tracked by the BrewBoard by showing all brews, filtering the brews by a recipe, adding notes to brews and deleting them
All the recipes and brews are stored in a PostgreSQL database.

## Used Technologies

Both parts (the GUI and the website) are implemented in the Rust programming language.
The website uses [axum](https://github.com/tokio-rs/axum) for the backend and routing and [sqlx](https://github.com/launchbadge/sqlx) for interfacing with the database.
The GUI uses [iced](https://github.com/iced-rs/iced).

## Current State
- [ ] Backend of the Website
- [ ] Prettying up the frontend
- [ ] GUI
- [ ] Connection to Bluetooth scales
- [ ] Integrating everything