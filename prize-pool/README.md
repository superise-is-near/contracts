# prize pool

## Development
1. Install rustup via https://rustup.rs/
2. Run the following:
```shell
rustup default stable
rustup target add wasm32-unknown-unknown
```

## Compiling
```shell
./scripts/build.sh
```

## Deploying to TestNet
To deploy to TestNet, you can run:

```shell
./scripts/dev-deploy.sh
```
This will output on the contract ID after finish deploy.



or you can change variables.sh,then run:
```shell
./scripts/deploy.sh
```