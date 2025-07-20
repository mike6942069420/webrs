# website

Git repository containing the webserver written in rust and webpage for [www.bigmike.ch](https://www.bigmike.ch)

> 🛡️ **License Notice**:  
> This project is licensed under the **GNU Affero General Public License v3.0 (AGPLv3)**.  
> If you deploy or modify this project and make it accessible over a network,  
> you **must** also make the complete, corresponding source code available to all users.
> 
> Please see the `LICENSE` file for the full **(AGPLv3)** license terms.
>
> The background photo `templates/images/bg.webp` included in this repository is licensed separately under  
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
| `make format_fix` | Format the code and automatically fix any issues using `cargo fmt -- --check` and `cargo clippy --fix` |
| `make deploy`   | Build the release webserver, build the Docker image, save it, and deploy it to the remote server using Docker Compose      |

##### Docker
The `Dockerfile` and `docker-compose.yml` files are a run environment for the webserver.\
The build still happens locally but the docker container is here to test the production binary.