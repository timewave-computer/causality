# Simulation System Test Suite

This document outlines the test suite for the causality simulation system. The test suite is designed to ensure that features work correctly in isolation and as part of the integrated system.

## Test Organization

The test suite is organized into several categories:

1. **Unit Tests**: Test individual components in isolation.
2. **Integration Tests**: Test the interaction between components.
3. **CLI Tests**: Test the command-line interface functionality.
4. **End-to-End Tests**: Test the entire system from user input to expected output.

## GeoRunner Tests (`geo_tests.rs`)

The GeoRunner is a new component that enables running simulations across multiple machines. The tests verify:

- Initialization and deployment of agents to remote hosts
- Starting and stopping scenarios across distributed machines
- Handling of SSH commands and error conditions
- Collection of logs from remote hosts
- Configuration management for different hosts

### Key Test Cases

- `test_geo_runner_initialize`: Verifies that the runner can initialize with a scenario, setting up the remote environment.
- `test_geo_runner_start`: Tests that agents can be started on remote machines.
- `test_geo_runner_stop`: Ensures that agents can be properly terminated and logs collected.
- `test_geo_runner_remote_error_handling`: Verifies that the system handles network errors gracefully.
- `test_geo_runner_host_specific_config`: Tests that host-specific configurations are applied correctly.
- `test_geo_runner_pause_resume`: Tests the pause and resume functionality across distributed hosts.

## Controller Tests (`controller_tests.rs`)

These tests focus on the controller's new capabilities for fact injection and agent state queries:

- Fact injection into running simulations
- Agent state querying
- Pause and resume functionality

### Key Test Cases

- `test_inject_fact`: Verifies that facts can be injected into a running simulation.
- `test_query_agent_state`: Tests that the state of an agent can be queried during simulation.
- `test_inject_fact_to_paused_scenario`: Ensures that fact injection fails when a simulation is paused.
- `test_query_agent_state_in_paused_scenario`: Verifies that agent state queries still work when a simulation is paused.
- `test_pause_resume_scenario`: Tests that scenarios can be paused and resumed.

## CLI Tests (`cli_tests.rs`)

The CLI tests ensure that the new commands added to the command-line interface work correctly:

- `inject-fact` command
- `query-agent` command
- `pause` and `resume` commands
- `logs` command with filtering options
- `list` and `status` commands

### Key Test Cases

- `test_inject_fact_command`: Tests injecting facts through the CLI.
- `test_query_agent_command`: Verifies agent state queries with different output formats.
- `test_pause_command` and `test_resume_command`: Test pause and resume functionality.
- `test_logs_command` and `test_logs_command_with_filtering`: Test log retrieval with various filtering options.
- `test_list_command`: Tests listing all running scenarios.
- `test_status_command`: Tests retrieving the status of a specific scenario.

## Running the Tests

To run all tests:

```bash
cargo test -p causality-simulation
```

To run specific test categories:

```bash
# Run GeoRunner tests
cargo test -p causality-simulation -- runner::geo_tests

# Run controller tests for fact injection
cargo test -p causality-simulation -- controller_tests::test_inject_fact

# Run CLI tests
cargo test -p causality-simulation -- cli_tests
```

## Test Dependencies

Most tests use mocking (via the `mockall` crate) to avoid actual network connections, file system operations, or other side effects. This makes the tests faster and more reliable.

For integration testing of the CLI, we use temporary files and directories (via the `tempfile` crate) to create test artifacts that are automatically cleaned up after the test completes.

## Testing Strategy

Our testing strategy focuses on:

1. **Mocking dependencies** to isolate the component being tested.
2. **Testing error cases** to ensure the system handles errors gracefully.
3. **Testing state transitions** (e.g., running → paused → running).
4. **Testing boundary conditions** (e.g., injecting facts into paused scenarios).

This comprehensive approach ensures that the simulation system is reliable and behaves as expected under various conditions. 