# Knockrs
Knockrs (pronounced knockers) is a reverse proxy written in rust using warp and tokio

## Usage
In config.json, add the port you would like knockrs to run on, and a dictionary containing domain names and the corresponding ports you would like to map them to. Check config.json for an example. Then use
```bash
cargo run
```
To run the project
## Note: paths don't work, imma fix that