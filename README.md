# website

Git repository containing the Rust webserver and webpage for [www.bigmike.ch](https://www.bigmike.ch).

> 🛡️ **License Notice**  
> This project is licensed under the **GNU Affero General Public License v3.0 (AGPLv3)**.  
> If you deploy or modify this project and make it accessible over a network,  
> you **must** also make the complete, corresponding source code available to all users.  
> See the `LICENSE` file for full license terms.
> 
> The background photo `templates/images/bg.webp` included in this repository is licensed separately under  
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
- Pushes changes to the remote repository with the correct tag

### Version Consistency

Before using `make git`, ensure the version string follows the format `vMAJOR.MINOR.PATCH` (e.g., `v1.1.1`) and only concerns codebase updates (`src` and `templates` directories). It must be the same across the following files:

- **`Makefile`**:  
  Set the `VERSION` variable to the current codebase version.

- **`Cargo.toml`**:  
  The `version` field should match the `Makefile` version.

- **`templates/index.html`**:  
  The `version-info` **div** should display the same version for deployment verification on the website.

*This process can be automated in the future.*
