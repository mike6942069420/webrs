# website

Git repository containing the Rust webserver and webpage for [www.bigmike.ch](https://www.bigmike.ch).

> 🛡️ **License Notice**  
> This project is licensed under the **GNU Affero General Public License v3.0 (AGPLv3)**.  
> If you deploy or modify this project and make it accessible over a network,  
> you **must** also make the complete, corresponding source code available to all users.  
> See the `LICENSE` file for full license terms.
> 
> The background photo `templates/bg.webp` included in this repository is licensed separately under  
> **Creative Commons Attribution 4.0 International (CC BY 4.0)**.  
> You may use and share the photo as long as proper attribution is provided.

## Development

Run `nix develop` to enter a shell with all required dependencies.

### Make Targets

| Command           | Description                                                                                  |
|-------------------|----------------------------------------------------------------------------------------------|
| `make`            | Build the release webserver                                                                  |
| `make run`        | Run the website locally in debug mode                                                        |
| `make create_dirs`| Create necessary directories for the webserver to run locally with `make run`              |
| `make release_run`| Build the release webserver and run it inside a docker container                            |
| `make clean`      | Clean build artifacts                                                                        |
| `make format`     | Format code using `cargo fmt` and run `cargo clippy`                                         |
| `make format_fix` | Format code and automatically fix issues using `cargo fmt -- --check` and `cargo clippy --fix` |
| `make deploy`     | Build release, create Docker image, save it, and deploy it to remote server via Docker Compose|
| `make git`        | Add all changes, prompt for commit message, and push commits with the correct tag            |
| `make full`       | Run `make git` and `make deploy` to commit changes and deploy the latest version            |

### Docker

The `Dockerfile` and `docker-compose.yml` provide a runtime environment for the webserver.  
The build happens locally; Docker is used to test the production binary in a containerized environment but it is also used for deployment by sending over the built Dockerfile image to the remote server.

### Git

The `make git` target is a convenience command that:

- Adds all changes to Git
- Prompts for a commit message
- Pushes changes to the remote repository with the correct tag representing the current version given in the `Makefile`, if the tag has not changed since last commit, it will not create a new tag.

### Version Consistency

Before using `make git`, ensure the package version in `Cargo.toml` follows the format `MAJOR.MINOR.PATCH` (e.g., `1.1.1`) and only concerns codebase updates (`src` and `templates` directories).