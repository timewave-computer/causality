# Causality CLI E2E Test Results

**Generated:** 2025-06-08 23:14:13 UTC

## 📊 Summary

| Metric | Value |
|--------|-------|
| Total Tests | 149 |
| Passed | 130 |
| Failed | 0 |
| Skipped | 19 |
| Success Rate | 87.2% |
| Total Duration | 2.50s |
| Average Test Time | 16.79ms |

## 📋 Results by Category

### ZK (zk)

- **Passed:** 21
- **Failed:** 0
- **Skipped:** 0
- **Success Rate:** 100.0%
- **Duration:** 396.39ms

### DEPLOY (deploy)

- **Passed:** 5
- **Failed:** 0
- **Skipped:** 0
- **Success Rate:** 100.0%
- **Duration:** 67.79ms

### ANALYZE (analyze)

- **Passed:** 10
- **Failed:** 0
- **Skipped:** 0
- **Success Rate:** 100.0%
- **Duration:** 120.73ms

### REPL (repl)

- **Passed:** 12
- **Failed:** 0
- **Skipped:** 0
- **Success Rate:** 100.0%
- **Duration:** 247.81ms

### PROJECT (project)

- **Passed:** 25
- **Failed:** 0
- **Skipped:** 0
- **Success Rate:** 100.0%
- **Duration:** 476.18ms

### HELP (help)

- **Passed:** 29
- **Failed:** 0
- **Skipped:** 6
- **Success Rate:** 82.9%
- **Duration:** 542.14ms

### TEST (test)

- **Passed:** 11
- **Failed:** 0
- **Skipped:** 0
- **Success Rate:** 100.0%
- **Duration:** 201.59ms

### INSPECT (inspect)

- **Passed:** 2
- **Failed:** 0
- **Skipped:** 0
- **Success Rate:** 100.0%
- **Duration:** 36.69ms

### VIZ (viz)

- **Passed:** 2
- **Failed:** 0
- **Skipped:** 0
- **Success Rate:** 100.0%
- **Duration:** 21.44ms

### CONFIG (config)

- **Passed:** 2
- **Failed:** 0
- **Skipped:** 0
- **Success Rate:** 100.0%
- **Duration:** 18.64ms

### DEV (dev)

- **Passed:** 11
- **Failed:** 0
- **Skipped:** 13
- **Success Rate:** 45.8%
- **Duration:** 124.39ms

## 🔧 Test Environment

- **OS:** macos
- **Architecture:** aarch64
- **CLI Version:** Not available

### Available Tools

- **rustc:** rustc 1.87.0 (17067e9ac 2025-05-09)
- **cargo:** cargo 1.87.0 (99624be96 2025-05-06)
- **ocaml:** The OCaml toplevel, version 5.1.1
- **dune:** 3.18.2
- **git:** git version 2.49.0

## 📝 Detailed Test Results

### ZK Tests

| Test Name | Status | Duration | Command |
|-----------|--------|----------|----------|
| zk_help | ✅ PASSED | 19.50ms | `causality zk --help` |
| zk_compile_help | ✅ PASSED | 23.40ms | `causality zk compile --help` |
| zk_compile_alias | ✅ PASSED | 13.99ms | `causality zk c --help` |
| zk_compile_privacy_low | ✅ PASSED | 21.64ms | `causality zk compile --input test.ir --output test_low.zk --privacy-level low` |
| zk_compile_privacy_medium | ✅ PASSED | 13.92ms | `causality zk compile --input test.ir --output test_medium.zk --privacy-level medium` |
| zk_compile_privacy_high | ✅ PASSED | 19.63ms | `causality zk compile --input test.ir --output test_high.zk --privacy-level high` |
| zk_compile_privacy_maximum | ✅ PASSED | 19.91ms | `causality zk compile --input test.ir --output test_maximum.zk --privacy-level maximum` |
| zk_compile_proof_system_groth16 | ✅ PASSED | 18.68ms | `causality zk compile --input test.ir --output test_groth16.zk --proof-system groth16` |
| zk_compile_proof_system_plonk | ✅ PASSED | 12.08ms | `causality zk compile --input test.ir --output test_plonk.zk --proof-system plonk` |
| zk_compile_proof_system_stark | ✅ PASSED | 35.29ms | `causality zk compile --input test.ir --output test_stark.zk --proof-system stark` |
| zk_compile_proof_system_marlin | ✅ PASSED | 16.48ms | `causality zk compile --input test.ir --output test_marlin.zk --proof-system marlin` |
| zk_compile_stats | ✅ PASSED | 22.00ms | `causality zk compile --input test.ir --output test_stats.zk --stats` |
| zk_prove_help | ✅ PASSED | 11.95ms | `causality zk prove --help` |
| zk_prove_basic | ✅ PASSED | 19.64ms | `causality zk prove --circuit test.zk --witness witness.json --output proof.zk` |
| zk_verify_help | ✅ PASSED | 11.40ms | `causality zk verify --help` |
| zk_verify_basic | ✅ PASSED | 23.15ms | `causality zk verify --circuit test.zk --proof proof.zk` |
| zk_verify_with_inputs | ✅ PASSED | 23.72ms | `causality zk verify --circuit test.zk --proof proof.zk --public-inputs public_inputs.json` |
| zk_verify_mock | ✅ PASSED | 14.45ms | `causality zk verify --circuit test.zk --proof proof.zk --mock` |
| zk_setup_help | ✅ PASSED | 9.34ms | `causality zk setup --help` |
| zk_setup_basic | ✅ PASSED | 25.07ms | `causality zk setup --circuit test.zk --output-dir setup_output` |
| zk_setup_multi_participants | ✅ PASSED | 20.33ms | `causality zk setup --circuit test.zk --output-dir setup_multi --participants 3` |

### DEPLOY Tests

| Test Name | Status | Duration | Command |
|-----------|--------|----------|----------|
| deploy_help | ✅ PASSED | 14.80ms | `causality deploy --help --help` |
| deploy_simulate_help | ✅ PASSED | 8.61ms | `causality deploy simulate --help` |
| deploy_submit_help | ✅ PASSED | 12.69ms | `causality deploy submit --help` |
| deploy_report_help | ✅ PASSED | 10.79ms | `causality deploy report --help` |
| deploy_simulate_chains | ✅ PASSED | 17.34ms | `causality deploy simulate --input test.ir --chains ethereum,polygon` |

### ANALYZE Tests

| Test Name | Status | Duration | Command |
|-----------|--------|----------|----------|
| analyze_help | ✅ PASSED | 12.95ms | `causality analyze --help --help` |
| analyze_code_help | ✅ PASSED | 7.33ms | `causality analyze code --help` |
| analyze_resources_help | ✅ PASSED | 9.03ms | `causality analyze resources --help` |
| analyze_effects_help | ✅ PASSED | 7.98ms | `causality analyze effects --help` |
| analyze_security_help | ✅ PASSED | 10.47ms | `causality analyze security --help` |
| analyze_alias | ✅ PASSED | 8.49ms | `causality a --help` |
| analyze_code_basic | ✅ PASSED | 10.25ms | `causality analyze code .` |
| analyze_resources_basic | ✅ PASSED | 9.97ms | `causality analyze resources -f test.lisp` |
| analyze_effects_basic | ✅ PASSED | 9.52ms | `causality analyze effects -f test.lisp` |
| analyze_security_basic | ✅ PASSED | 34.33ms | `causality analyze security .` |

### REPL Tests

| Test Name | Status | Duration | Command |
|-----------|--------|----------|----------|
| repl_basic_help | ✅ PASSED | 36.65ms | `causality repl --help` |
| repl_debug_help | ✅ PASSED | 16.59ms | `causality repl --debug --help` |
| repl_show_state_help | ✅ PASSED | 29.11ms | `causality repl --show-state --help` |
| repl_max_steps_help | ✅ PASSED | 12.85ms | `causality repl --max-steps 100 --help` |
| repl_load_tutorial_basic | ✅ PASSED | 12.97ms | `causality repl --load-tutorial basic --help` |
| repl_load_tutorial_effects | ✅ PASSED | 13.80ms | `causality repl --load-tutorial effects --help` |
| repl_load_tutorial_zk | ✅ PASSED | 26.90ms | `causality repl --load-tutorial zk --help` |
| repl_load_tutorial_defi | ✅ PASSED | 25.46ms | `causality repl --load-tutorial defi --help` |
| repl_auto_save_help | ✅ PASSED | 20.70ms | `causality repl --auto-save --help` |
| repl_alias | ✅ PASSED | 18.96ms | `causality r --help` |
| repl_invalid_tutorial | ✅ PASSED | 17.07ms | `causality repl --load-tutorial nonexistent --help` |
| repl_combined_options | ✅ PASSED | 16.22ms | `causality repl --debug --show-state --max-steps 50 --help` |

### PROJECT Tests

| Test Name | Status | Duration | Command |
|-----------|--------|----------|----------|
| project_help | ✅ PASSED | 15.51ms | `causality project --help` |
| project_alias | ✅ PASSED | 11.09ms | `causality p --help` |
| project_new_help | ✅ PASSED | 16.62ms | `causality project new --help` |
| project_new_basic | ✅ PASSED | 37.09ms | `causality project new test-basic-project --template basic` |
| project_new_defi | ✅ PASSED | 24.93ms | `causality project new test-defi-project --template defi` |
| project_new_privacy | ✅ PASSED | 29.74ms | `causality project new test-privacy-project --template privacy` |
| project_new_zk | ✅ PASSED | 31.57ms | `causality project new test-zk-project --template zk` |
| project_new_library | ✅ PASSED | 29.60ms | `causality project new test-library-project --template library` |
| project_new_advanced | ✅ PASSED | 20.78ms | `causality project new test-advanced-project --template advanced` |
| project_new_with_git | ✅ PASSED | 21.82ms | `causality project new git-test-project --template basic --git` |
| project_new_with_description | ✅ PASSED | 27.92ms | `causality project new desc-test-project --template basic --description A test project with description` |
| project_init_help | ✅ PASSED | 19.85ms | `causality project init --help` |
| project_init_empty | ✅ PASSED | 30.84ms | `causality project init` |
| project_init_force | ✅ PASSED | 19.85ms | `causality project init --force` |
| project_build_help | ✅ PASSED | 14.53ms | `causality project build --help` |
| project_build_alias | ✅ PASSED | 19.47ms | `causality project b --help` |
| project_build_release | ✅ PASSED | 11.53ms | `causality project build --release --help` |
| project_build_timings | ✅ PASSED | 13.85ms | `causality project build --timings --help` |
| project_clean_help | ✅ PASSED | 11.67ms | `causality project clean --help` |
| project_clean_deep | ✅ PASSED | 10.49ms | `causality project clean --deep --help` |
| project_status_help | ✅ PASSED | 13.04ms | `causality project status --help` |
| project_status_alias | ✅ PASSED | 9.42ms | `causality project s --help` |
| project_status_deps | ✅ PASSED | 12.87ms | `causality project status --deps --help` |
| project_add_help | ✅ PASSED | 10.94ms | `causality project add --help` |
| project_add_with_version | ✅ PASSED | 10.70ms | `causality project add test-package --version 1.0.0 --help` |

### HELP Tests

| Test Name | Status | Duration | Command |
|-----------|--------|----------|----------|
| help_basic | ✅ PASSED | 15.83ms | `causality help` |
| help_short_flag | ✅ PASSED | 26.11ms | `causality -h` |
| help_long_flag | ✅ PASSED | 42.56ms | `causality --help` |
| help_tutorial | ⏭️ SKIPPED | 0.00ns | `causality help tutorial` |
| help_guides | ⏭️ SKIPPED | 0.00ns | `causality help guides` |
| help_reference | ⏭️ SKIPPED | 0.00ns | `causality help reference` |
| help_examples | ⏭️ SKIPPED | 0.00ns | `causality help examples` |
| help_api | ⏭️ SKIPPED | 0.00ns | `causality help api` |
| help_troubleshooting | ⏭️ SKIPPED | 0.00ns | `causality help troubleshooting` |
| help_repl_command | ✅ PASSED | 23.06ms | `causality repl --help` |
| help_project_command | ✅ PASSED | 14.97ms | `causality project --help` |
| help_dev_command | ✅ PASSED | 20.18ms | `causality dev --help` |
| help_zk_command | ✅ PASSED | 19.10ms | `causality zk --help` |
| help_deploy_command | ✅ PASSED | 31.90ms | `causality deploy --help` |
| help_analyze_command | ✅ PASSED | 26.47ms | `causality analyze --help` |
| help_test_command | ✅ PASSED | 12.60ms | `causality test --help` |
| help_inspect_command | ✅ PASSED | 17.30ms | `causality inspect --help` |
| help_viz_command | ✅ PASSED | 13.44ms | `causality viz --help` |
| help_config_command | ✅ PASSED | 15.51ms | `causality config --help` |
| help_project_new | ✅ PASSED | 14.32ms | `causality project new --help` |
| help_project_build | ✅ PASSED | 31.90ms | `causality project build --help` |
| help_project_status | ✅ PASSED | 9.62ms | `causality project status --help` |
| help_dev_compile | ✅ PASSED | 15.32ms | `causality dev compile --help` |
| help_dev_run | ✅ PASSED | 14.82ms | `causality dev run --help` |
| help_dev_serve | ✅ PASSED | 18.15ms | `causality dev serve --help` |
| help_zk_compile | ✅ PASSED | 16.42ms | `causality zk compile --help` |
| help_zk_prove | ✅ PASSED | 14.69ms | `causality zk prove --help` |
| help_zk_verify | ✅ PASSED | 15.04ms | `causality zk verify --help` |
| help_deploy_simulate | ✅ PASSED | 16.93ms | `causality deploy simulate --help` |
| help_deploy_submit | ✅ PASSED | 21.12ms | `causality deploy submit --help` |
| help_analyze_code | ✅ PASSED | 13.57ms | `causality analyze code --help` |
| help_analyze_resources | ✅ PASSED | 10.46ms | `causality analyze resources --help` |
| help_test_unit | ✅ PASSED | 20.81ms | `causality test unit --help` |
| help_test_e2e | ✅ PASSED | 16.72ms | `causality test e2e --help` |
| help_invalid_topic | ✅ PASSED | 10.87ms | `causality help nonexistent` |

### TEST Tests

| Test Name | Status | Duration | Command |
|-----------|--------|----------|----------|
| test_help | ✅ PASSED | 13.85ms | `causality test --help` |
| test_alias | ✅ PASSED | 16.05ms | `causality t --help` |
| test_unit_help | ✅ PASSED | 10.45ms | `causality test unit --help` |
| test_effects_help | ✅ PASSED | 22.96ms | `causality test effects --help` |
| test_integration_help | ✅ PASSED | 21.67ms | `causality test integration --help` |
| test_e2e_help | ✅ PASSED | 18.22ms | `causality test e2e --help` |
| test_integration_alias | ✅ PASSED | 16.57ms | `causality test int --help` |
| test_unit_basic | ✅ PASSED | 35.99ms | `causality test unit` |
| test_unit_coverage | ✅ PASSED | 10.54ms | `causality test unit --coverage` |
| test_effects_property | ✅ PASSED | 24.15ms | `causality test effects --property-based` |
| test_e2e_chains | ✅ PASSED | 10.88ms | `causality test e2e --chains ethereum,polygon` |

### INSPECT Tests

| Test Name | Status | Duration | Command |
|-----------|--------|----------|----------|
| inspect_help | ✅ PASSED | 13.60ms | `causality inspect --help` |
| inspect_alias | ✅ PASSED | 23.05ms | `causality i --help` |

### VIZ Tests

| Test Name | Status | Duration | Command |
|-----------|--------|----------|----------|
| viz_help | ✅ PASSED | 12.58ms | `causality viz --help` |
| viz_alias | ✅ PASSED | 8.83ms | `causality v --help` |

### CONFIG Tests

| Test Name | Status | Duration | Command |
|-----------|--------|----------|----------|
| config_help | ✅ PASSED | 10.71ms | `causality config --help` |
| config_alias | ✅ PASSED | 7.90ms | `causality c --help` |

### DEV Tests

| Test Name | Status | Duration | Command |
|-----------|--------|----------|----------|
| dev_help | ✅ PASSED | 13.62ms | `causality dev --help` |
| dev_alias | ✅ PASSED | 9.26ms | `causality d --help` |
| dev_compile_help | ✅ PASSED | 8.61ms | `causality dev compile --help` |
| dev_compile_alias | ✅ PASSED | 10.77ms | `causality dev c --help` |
| dev_compile_intermediate | ⏭️ SKIPPED | 0.00ns | `causality dev compile -i test.lisp -o test.intermediate --format intermediate` |
| dev_compile_bytecode | ⏭️ SKIPPED | 0.00ns | `causality dev compile -i test.lisp -o test.bytecode --format bytecode` |
| dev_compile_native | ⏭️ SKIPPED | 0.00ns | `causality dev compile -i test.lisp -o test.native --format native` |
| dev_compile_wasm | ⏭️ SKIPPED | 0.00ns | `causality dev compile -i test.lisp -o test.wasm --format wasm` |
| dev_compile_js | ⏭️ SKIPPED | 0.00ns | `causality dev compile -i test.lisp -o test.js --format js` |
| dev_compile_optimize | ⏭️ SKIPPED | 0.00ns | `causality dev compile -i test.lisp -o test_opt.ir --optimize` |
| dev_compile_show_stages | ⏭️ SKIPPED | 0.00ns | `causality dev compile -i test.lisp -o test_stages.ir --show-stages` |
| dev_run_help | ✅ PASSED | 10.71ms | `causality dev run --help` |
| dev_run_alias | ✅ PASSED | 12.81ms | `causality dev r --help` |
| dev_run_file | ⏭️ SKIPPED | 0.00ns | `causality dev run -f test.lisp` |
| dev_run_source | ⏭️ SKIPPED | 0.00ns | `causality dev run -s (+ 1 2)` |
| dev_run_trace | ⏭️ SKIPPED | 0.00ns | `causality dev run -f test.lisp --trace` |
| dev_run_max_steps | ⏭️ SKIPPED | 0.00ns | `causality dev run -f test.lisp --max-steps 1000` |
| dev_serve_help | ✅ PASSED | 10.55ms | `causality dev serve --help` |
| dev_serve_port | ✅ PASSED | 10.22ms | `causality dev serve --port 8080 --help` |
| dev_serve_watch | ✅ PASSED | 8.92ms | `causality dev serve --watch --help` |
| dev_serve_open | ✅ PASSED | 12.83ms | `causality dev serve --open --help` |
| dev_fmt_help | ✅ PASSED | 15.48ms | `causality dev fmt --help` |
| dev_fmt_check | ⏭️ SKIPPED | 0.00ns | `causality dev fmt --check` |
| dev_fmt_files | ⏭️ SKIPPED | 0.00ns | `causality dev fmt test.lisp` |

