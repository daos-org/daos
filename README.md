# daos

## Introduce
We provide an abstraction layer to make it easier for each DAO to manage each related specific transaction, and how to quickly create a DAO based on daos.
***
* As a developer, you can just focus on the design of the DAO template, leave the governance rules to us.
* As a user, you can create any number of daos for yourself based on the DAO template.
* As an eco-builder, this can help you really create groups, focus on sinking markets and hear small but meaningful voices. Help you achieve greater decentralization.
* Each DAO can perform external transactions as an ordinary user through intra-DAO governance.
* The most important is each DAO's governance can evolve on its own.

## Develop
* rust
`curl https://sh.rustup.rs -sSf | sh`
* Clone Project From Github  

`git clone https://github.com/DAOS-TEAM/daos.git`
* Build  

```angular2html
cd daos
cargo build --release
```
## Test
```asm
cargo test
```
If you want to see the test coverage report
```asm
cargo install cargo-tarpaulin
cargo tarpaulin --out html --run-types Tests
```
## Docs
```asm
cargo doc --open
```
## Benchmarking
```asm
git clone https://github.com/DICO-TEAM/dico-chain.git
cd dico-chain
./scripts/daos_benchmarkall.shdia
```

## [Workflow](./document/workflow.md)
## License

The project is made available under the [Apache2.0](./LICENSE-APACHE2) license.

## Projects using daos
* If you intend or are using daos, please add your project here  

In alphabetical order
* [Listen Network](https://github.com/listenofficial/listen-parachain)
* [KICO](https://github.com/DICO-TEAM/dico-chain)
