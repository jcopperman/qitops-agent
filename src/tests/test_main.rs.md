 This code appears to be a part of a CLI (Command Line Interface) tool or script for managing a monitoring system, possibly for containerized applications like Docker. It defines several functions such as:

1. `main()` - the entry point for the program, handling command-line arguments and dispatching them to other functions.
2. Various helper functions for printing messages and handling user input (`print_welcome`, `print_menu`, etc.).
3. Functions related to interacting with Docker, such as starting, stopping, and checking the status of a monitoring stack defined in `docker-compose-monitoring.yml`.
4. Error handling functions using `anyhow` crate for reporting errors to the user.
5. Utility functions like `ProgressIndicator`, which displays progress messages during long-running tasks.
6. The code seems to use the `indicatif` crate for creating simple console UI elements, such as progress bars.
7. It appears that the monitoring stack includes a Grafana instance, accessed at `http://localhost:3000`, with default credentials of `admin/qitops`.
8. The program seems to support command-line options like `--docker` when starting the monitoring stack.