`fast_config`
=============
---

[<img alt="github" src="https://img.shields.io/badge/github-fast_config-brightgreen.svg?logo=github&style=for-the-badge"/>](https://github.com/FlooferLand/fast_config)
[<img alt="crates.io" src="https://img.shields.io/crates/v/fast_config?logo=rust&style=for-the-badge"/>](https://crates.io/crates/fast_config)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-fast_config-988.svg?logo=rust&style=for-the-badge"/>](https://docs.rs/fast_config)
<br style="display: block; margin: 0 0; content: '---'" />
[<img alt="license" src="https://img.shields.io/github/license/FlooferLand/fast_config?style=flat"/>](https://github.com/FlooferLand/fast_config/blob/main/LICENSE)
[<img alt="code size" src="https://img.shields.io/github/languages/code-size/FlooferLand/fast_config?style=flat"/>](https://www.youtube.com/watch?v=dQw4w9WgXcQ)
[<img alt="issues" src="https://img.shields.io/github/issues/FlooferLand/fast_config?label=open%20issues&style=flat"/>](https://github.com/FlooferLand/fast_config/issues)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/FlooferLand/fast_config/main_test.yml)

A small, safe, lightweight, and easy-to-use Rust crate to read and write to config files.

Currently only supports:
[JSON](https://crates.io/crates/serde_json) & [JSON5](https://crates.io/crates/json5), [TOML](https://crates.io/crates/toml),  and [YAML](https://crates.io/crates/serde_yml).

But more [Serde](https://serde.rs/)-supported formats *(such as RON)* are planned to be added later.

### Useful teleports:
- [Migrating to a newer version of the crate](https://github.com/FlooferLand/fast_config/blob/main/CONVERSION_TUTORIAL.md)
- [Code examples](#examples)
- [Getting Started](#getting-started)
- [Things that need work (for contributors!)](./CONTRIBUTORS.md)

## What is this crate?

`fast_config` was made to be a faster to set up, more light-weight, statically typed alternative to [config](https://crates.io/crates/config).

It also manages to have its own benefits compared to some other config-reading crates
as there is full support for writing/saving config files,
and it also provides you with *some* options regarding styling your config files

---

### Why this crate?
- It's small and fast *(uses compile-time features to remove/add code)*
- It's safe and robust *(uses Rust's structs to store data, instead of HashMaps)*
- Ridiculously simple to use *(only takes 3 lines of short code to make a config file, write/read something, and save it)*

### Why not this crate?
1. It doesn't work if you don't know the way your data will be formatted<br>
   *(for example if you want your users to be able to have any keys ranging from `key0` to `key9000` in an object)*
2. It cannot currently understand the RON file format
3. It cannot currently save comments in config files.

---

**2** and **3** _are_ going to be addressed with future updates, however.

### ⚠ Documentation and tests are still being made! ⚠
This crate is now stable, I however haven't battle-tested this in any humongous projects,
so while there will NOT be any panics or crashes, some weird things might happen at scale.

Documentation might be a little weird or incomplete at the current moment, too.

Feel free to contribute any fixes by [opening up an issue](https://github.com/FlooferLand/fast_config/issues) if you find
anything that isn't working as expected!

---

## Examples:
```rust
use fast_config::Config;
use serde::{Serialize, Deserialize};

// Creating a config struct to store our data
#[derive(Serialize, Deserialize)]
pub struct MyData {
    pub student_debt: i32,
}

fn main() {
    // Initializing a logging system (needed to show some warnings/errors)
    env_logger::init();

    // Creating our data (default values)
    let data = MyData {
        student_debt: 20,
    };

    // Creating a new config struct with our data struct
    let mut config = Config::new("./config/myconfig.json5", data).unwrap();

    // Read/writing to the data
    println!("I am ${} in debt", config.data.student_debt);
    config.data.student_debt = i32::MAX;
    println!("Oh no, i am now ${} in debt!!", config.data.student_debt);

    // Saving it back to the disk
    config.save().unwrap();
}
```

## Getting started

1. Add the crate to your project via <br/> `cargo add fast_config`
   - Additionally, also add `serde` as it is required!

2. Enable the feature(s) for the format(s) you'd like to use <br/>
   - Currently only `json5`, `toml`, and `yaml` are supported <br/>

3. Create a struct to hold your data that derives `Serialize` and `Deserialize`

4. Create an instance of your data struct
   - Optionally `use` the crate's `Config` type for convenience: `use fast_config::Config;`

5. To create and store your config file(s), use:
   ```rust,ignore
   let my_config = Config::new("./path/to/my_config_file", your_data).unwrap();
   ```
    Alternatively, you can use `Config::from_settings` to style some things and manually set the format!

---

View the [examples](./examples) directory for more advanced examples.

## NOTE: This project will be rewritten sometime
The code is currently very messy, but I'm too busy with other projects to deal with it. </br>
I've improved a lot as a Rust developer since the creation of this project and a lot of the ways you interface with it could be better.

Some things I want to do for the rewrite are listed in a comment at the top of [lib.rs](./src/lib.rs)
Some other ideas I'll have to experiment with:
- Moving to a trait-based approach where you can slap a `#[derive(FastConfig)]` onto any struct to give it the `save`/`load` functions.
  This makes the annoying `my_config.data.my_setting` into simply `my_config.my_setting`

A conversion guide for the rewrite will be available, as I'll have to convert over my projects as well to use the rewritten `fast_config`.

The rewrite should be smaller, safer, and the source code will most importantly be ***way more readable***.

---
<br/>
