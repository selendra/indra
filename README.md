# Selendra-parachain
A set of tools for writing Substrate-based Selendra parachains

## Local Setup
Launch a local setup including a Relay Chain and a Parachain.

### Launch the Relay Chain
```sh
# Clone and compile Selendra
git clone https://github.com/selendra/selendra-chain
cargo build --release

# Generate a raw chain spec
./target/release/selendra build-spec --chain cardamom-local --disable-default-bootnode --raw > cardamom-local.json

# Alice
./target/release/selendra --chain cardamom-local.json --alice --tmp --ws-port 9945

# Bob (In a separate terminal)
./target/release/selendra --chain cardamom-local.json --bob --tmp --ws-port 9946 --port 30334
```
For more information see [here](https://github.com/selendra/selendra-chain/blob/main/README.md).

### Launch the Parachain

#### Launcha Parachain with Alice

```sh
# Compile 
git clone https://github.com/selendra/selendra-parachain-template
cargo build --release

# Export genesis state
# parachain-id 2000 as default at local 
./target/release/parachain-collator export-genesis-state --chain local > genesis-state

# Export genesis wasm
./target/release/parachain-collator export-genesis-wasm --chain local > genesis-wasm

# Launch Alice
# It also assumes. Change as needed. parachain-id 2000 as default at local 
./target/release/parachain-collator \
-d /tmp/parachain/alice \
--collator \
--alice \
--chain local \
--base-path parachain/alice \
--force-authoring \
--ws-port 8844 \
--port 40333 \
--rpc-port 8833 \
-- \
--execution wasm \
--chain ../selendra-chain/cardamom-local.json

# Launch Bob
# It also assumes. Change as needed. parachain-id 2000 as default at local 
./target/release/parachain-collator \
-d /tmp/parachain/alice \
--collator \
--bob \
--chain local \
--base-path parachain/bob \
--force-authoring \
--ws-port 8845 \
--port 40334 \
--rpc-port 8834 \
-- \
--execution wasm \
--chain ../selendra-chain/cardamom-local.json

```

#### Launch Parachain with custom chain spec

Create a custom chain spec. Now that each participant has their own keys, you're ready to create a custom chain specification. We will use this custom chain spec instead of the built-in local spec that we used previously.

##### Create a chain specification

Last time around, we used --chain local which is a predefined "chain spec" that has Alice and Bob specified as validators along with many other useful defaults.

Rather than writing our chain spec completely from scratch, we'll just make a few modifications to the one we used before. To start, we need to export the chain spec to a file named customSpec.json.

```sh
./target/release/parachain-collator build-spec --disable-default-bootnode --chain local > customSpec.json
```

##### Modify Aura authority nodes

The file we just created contains several fields, and you can learn a lot by exploring them. By far the largest field is a single binary blob that is the Wasm binary of our runtime. It is part of what you built earlier when you ran the cargo build --release command.

```json
    "session": {
        "keys": [
          [
            "5CDLd7jXEm8u11MzqMPNWenHQGbSKPMREA29XsRNp293Frxw",
            "5CDLd7jXEm8u11MzqMPNWenHQGbSKPMREA29XsRNp293Frxw",
            {
              "aura": "5CDLd7jXEm8u11MzqMPNWenHQGbSKPMREA29XsRNp293Frxw"
            }
          ],
          [
            "5EuwnnxtbdDyBd4keWJjJ7vY8H9PE6BHwWje3tP3qLMMUwZo",
            "5EuwnnxtbdDyBd4keWJjJ7vY8H9PE6BHwWje3tP3qLMMUwZo",
            {
              "aura": "5EuwnnxtbdDyBd4keWJjJ7vY8H9PE6BHwWje3tP3qLMMUwZo"
            }
          ]
        ]
    },
```

All we need to do is change the authority addresses listed (currently Alice and Bob) to our own addresses.

#### Convert to raw chain spec

Once the chain spec is prepared, convert it to a "raw" chain spec. The raw chain spec contains all the same information, but it contains the encoded storage keys that the node will use to reference the data in its local storage. Distributing a raw spec ensures that each node will store the data at the proper storage keys.

```sh
./target/release/parachain-collator build-spec --disable-default-bootnode --chain=customSpec.json --raw > customSpecRaw.json
```

With your custom chain spec created, you're ready to launch your own custom chain.

#### Launch your network

You've completed all the necessary prep work and you're now ready to launch your chain. This process is very similar to when you launched a chain earlier, as Alice and Bob. It's important to start with a clean base path, so if you plan to use the same path that you've used previously, please delete all contents from that directory.

```sh
# Start node
./target/release/parachain-collator \
-d /tmp/parachain/all \
--collator \
--force-authoring \
--chain=customSpecRaw.json \
--port 40337 \
--ws-port 9947 \
-- \
--execution wasm \
--chain ../selendra-chain/selendra-local.json
```

#### Add keys to keystore

Once your node is running, you will again notice that no blocks are being produced. At this point, you need to add your keys into the keystore.

You can use the Apps UI to insert your keys into the keystore. Navigate to "Developer --> RPC Call". Choose "author" and "insertKey". The fields can be filled like this:

```
keytype: aura
suri: <your mnemonic phrase> (eg. clip organ olive upper oak void inject side suit toilet stick narrow)
publicKey: <your raw sr25519 key> (eg. 0x9effc1668ca381c242885516ec9fa2b19c67b6684c02a8a3237b6862e5c8cd7e)
```

Congratulations! You've started your own Parachain!

### Parachain Registration
Now that you have two relay chain nodes, and a parachain node accompanied with a relay chain light client running, the next step is to register the parachain in the relay chain with the following steps.
- Goto UI, connecting to your relay chain.
- Execute a sudo extrinsic on the relay chain by going to Developer -> sudo page.
- Pick paraSudoWrapper -> sudoScheduleParaInitialize(id, genesis) as the extrinsic type, shown below.

- Set the id: ParaId to 2000 (or whatever ParaId you used above), and set the parachain: Bool option to Yes.

- For the genesisHead, drag the genesis state file exported above, para-state, in.

- For the validationCode, drag the genesis wasm file exported above, para-wasm, in.

### Restart the Parachain (Collator)

The collator node may need to be restarted to get it functioning as expected. After a new epoch starts on the relay chain, your parachain will come online. Once this happens, you should see the collator start reporting parachain blocks.

### Perform Runtime upgade

We need to inform the relay chain we will be running an upgrade, so we send them a hash of our wasm blob via a sudo extrinsic call from our parachain

```sh
sudo > parachainSystem > authorizeUpgrade
```

Once that has been submitted correctly then:

```sh
Extrinsics > parachainSystem > enactAuthorizedUpgrade
```

If you increased the spec version, you will see the new spec version in the top left of your polkadot-js apps UI screen.

## License

Selendra-parachain is implement from [Cumulus](https://github.com/paritytech/cumulus.git) under license [GPL 3.0 licensed](LICENSE-GPL3).
