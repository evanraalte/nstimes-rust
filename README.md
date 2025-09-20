# NSTimes

NS travel advice in your terminal!

## Obtain a token
Create an account at the [NS API portal](https://apiportal.ns.nl/signin). Then create a token [here](https://apiportal.ns.nl/api-details#api=reisinformatie-api).

put it in an `.env` file as `NS_API_TOKEN=***` or export it with `export NS_API_TOKEN=***`

Simply run e.g. :
```bash
cargo run  "Den Haag C" "Amersfoort C" 
```

To build a release:

```bash
cargo build --release   
```