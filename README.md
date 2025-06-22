# website

Git repository containing the webserver written in rust and webpage for [www.bigmike.ch](https://www.bigmike.ch)

> 🛡️ **License Notice**:  
> This project is licensed under the **GNU Affero General Public License v3.0 (AGPLv3)**.  
> If you deploy or modify this project and make it accessible over a network,  
> you **must** also make the complete, corresponding source code available to all users.
> 
> Please see the `LICENSE` file for the full **(AGPLv3)** license terms.
>
> The background photo `bg.webp` included in this repository is licensed separately under  
> **Creative Commons Attribution 4.0 International (CC BY 4.0)**.  
> You are free to use and share the photo as long as you provide proper attribution.

---

### Development

Run `nix develop` to enter a shell with all required dependencies.

#### Make Targets

| Command         | Description                                                                                   |
|-----------------|-----------------------------------------------------------------------------------------------|
| `make`          | Build the full release webserver                                                              |
| `make run`      | Run the website locally for debugging                                                         |
| `make release_run` | Build the release webserver and run it                                                        |
| `make clean`    | Clean the build directory                                                                     |
| `make format`   | Format the code using `cargo fmt` and `cargo clippy`                                          |
| `make test`     | Run tests                                                                                     |