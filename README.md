# The Legacy Rust Toolset

## Why this is here

This is where I got with the lattice by using procedural programming. This has been supersceeded by the Computational Reality Engine. This is just to prove the basics of the lattice and how that works. Technically, the `usherd` is capable of being a rudimentary node, although there's been a slight tweak to the Rhex data structure (Context changed to `spacial_ref` & `spacial_data` fields instead of the x,y,z,refer setup from before)

## Please don't use

This is literally here just til the CRE becomes lattice aware. If you're looking for references on building transforms I strongly point you to the documentation for now. This is going to give you an idea how the crypto works but like, again, this is a completely different ballgame from the CRE.

## How to run

### Compile

Obviously this needs compiling. `cargo build` in the root dir of the project should suffice. Binaries will of course be in `./target/build/`

### Keys

You will need at least one key to get started. I recommend since **you aren't** using this for production (right?) that you just make a hot key file rather than fucking with a password

```bash
./target/debug/keytool generate \
    -v \
    --hot \
    --keyfile ../secrets/master.key
```

This is what you will need for the most of everything.

### Setup

#### Config File

The `usherd_config.json` is the main config file for the **usherd** daemon. It looks like the following:

```json
{
    "host": "localhost",
    "port": 1984,
    "bin_dir": "./target/debug",
    "fs_dir": "./ledger/fs",
    "data_dir": "./",
    "cache_db": "./ledger/cache/cache.db",
    "hot_keys": ["../secrets/master.key"],
    "verbose": true
}
```

Make sure your key is listed as a "hot_keys" member. This is essential for the system to correlate your key to submissions later.

#### Dirs

The directory structure used it totally up to you of course, but it does have to exist (the code doesn't create them for you).

#### Cache

The cache database has to be generated. Use:

```bash
./target/debug/usherd rebuild \
    -v \
    --config ./usherd_config.json
```

This creates the blank cache SQLite DB file with the appropriate tables.

#### Genesis

Every lattice has to have an origin. To create one, use the genesis tool. The command is something like:

```bash
./target/debug/genesis create \
    -v \
    --keyfile ../secrets/master.key \
    --output ./ledger/fs
```

This drops a `genesis.rhex` signed by the key into the ledger's file system. To view the `current_hash` value for the genesis record (will be needed to append a policy) just use the `rhex-craft` tool

```bash
./target/debug/rhex-craft view \
    --input ./ledger/fs/genesis.rhex
```

#### Listening

If you've completed these steps so far, techincally the lattice is created, but like... the point is to add to it. From here you can run:

```bash
./target/debug/usherd listen \
    -v \
    --port 1984 \
    --host localhost \
    --config ./usherd_config.json
```

This should spin up the usher daemon. Once running it will be hosting the genesis record. To append anything at this point you have to append a policy signed by the master hot key you created. That's a whole other document itself.

## Appending

### Crafting

Rhex are complicated little buggers. They take a series of fields and a data file to create the CBOR object to be hashed and signed.

To make the new policy, or any other record really, you will need to prep a JSON file with the data payload intended for the Rhex. From there you will need the base64 url safe encoded public key of the author key, the base64 url safe encoded public key of the receiving usher, the current hash of the record you are appending after, in the form of `--previous-hash`, again in base64 url safe encoding.

A sample command might look like:

```bash
./target/debug/rhex-craft craft \
    --output ./test.rhex \
    --scope "" \
    --author-pk q84dC-_uIKPhsDbx-WNeMgYYOUEViKTOuyytO8oKyQI \
    --usher-pk q84dC-_uIKPhsDbx-WNeMgYYOUEViKTOuyytO8oKyQI \
    --record-type "policy:set" \
    --data ./policy-set.json \
    --previous-hash 1lUFQQHCn3zsBUt7a8v6h9FPCVtx-ancoQk3Fhk_H5Y`
```

**Note:** The provided `policy-set.json` will set a basic scope policy to tinker with.

### Signing

Each record has to have the appropriate signatures. First the Author, by signing with the `keytool`, as follows:

```bash
./target/debug/keytool sign \
    -v \
    --keyfile ../secrets/master.key \
    --hot \
    --sig-type author \
    --input ./ledger/fs/test.rhex \
    --output ./
```

Pay attention to the fact that the `--output` is the dir it will land in. It's automatically named the Crockford base32 of the `previous-hash` in that dir. This allows for using the `keytool` to sign your way into Rhex on the disk without having to submit them, which is honestly sketch as hell, but this is a prototype, remember? lol

For a complete record you will need to repeat the procedure for the Usher and any Quorum members.

### Finalizing

Once all the signatures are done, you can finalize the Rhex, which just stamps it with the `current_hash` field that is the Blake3 of the CBOR object. This actually takes a filename as it's output. I don't remember why. lol

```bash
./target/debug/rhex-craft finalize \
    --input ./ledger/fs/test.rhex \
    --output ./send.rhex
```

This is required for submission. This is the unique coordinate (plus the `at` time) of the Rhex.

### Submission

After finalization, submission is just a matter of using the `usher` client tool.

```bash
./target/debug/usher submit \
    --host localhost \
    --port 1984 \
    --input ./send.rhex
```

It will of course either accept or reject based on the policy you created.

### Viewing

To view an existing Rhex stored on disk, use the `rhex-craft` tool.

```bash
./target/debug/rhex-craft view \
    --input ./ledger/fs/genesis.rhex
```

## Requesting

A request is actually just an append request for an ephemeral record_type like "request:rhex". This *can* be done in the `rhex-craft` tool, but the `usher` client provides a much easier interface.

```bash
./target/debug/usher request \
    --host localhost \
    --port 1984 \
    --keyfile ../secrets/master.key \
    --scope ""
```

This creates a request for all the rhex belonging to the scope "root.node.leaf" and submits it.

In this implementation you recieve a duplicate of your request back, as an 'echo' and then the results of the request.
