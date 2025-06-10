# Causality CLI E2E Test Results

**Generated:** 2025-06-10 17:41:34 UTC

## 📊 Summary

| Metric | Value |
|--------|-------|
| Total Tests | 149 |
| Passed | 130 |
| Failed | 0 |
| Skipped | 19 |
| Success Rate | 87.2% |
| Total Duration | 1.08s |
| Average Test Time | 7.26ms |

## 📋 Results by Category

### ZK (zk)

- **Passed:** 21
- **Failed:** 0
- **Skipped:** 0
- **Success Rate:** 100.0%
- **Duration:** 155.72ms

### REPL (repl)

- **Passed:** 12
- **Failed:** 0
- **Skipped:** 0
- **Success Rate:** 100.0%
- **Duration:** 85.45ms

### DEV (dev)

- **Passed:** 11
- **Failed:** 0
- **Skipped:** 13
- **Success Rate:** 45.8%
- **Duration:** 73.18ms

### DEPLOY (deploy)

- **Passed:** 5
- **Failed:** 0
- **Skipped:** 0
- **Success Rate:** 100.0%
- **Duration:** 32.02ms

### ANALYZE (analyze)

- **Passed:** 10
- **Failed:** 0
- **Skipped:** 0
- **Success Rate:** 100.0%
- **Duration:** 146.27ms

### PROJECT (project)

- **Passed:** 25
- **Failed:** 0
- **Skipped:** 0
- **Success Rate:** 100.0%
- **Duration:** 188.99ms

### HELP (help)

- **Passed:** 29
- **Failed:** 0
- **Skipped:** 6
- **Success Rate:** 82.9%
- **Duration:** 218.35ms

### INSPECT (inspect)

- **Passed:** 2
- **Failed:** 0
- **Skipped:** 0
- **Success Rate:** 100.0%
- **Duration:** 11.22ms

### VIZ (viz)

- **Passed:** 2
- **Failed:** 0
- **Skipped:** 0
- **Success Rate:** 100.0%
- **Duration:** 13.64ms

### CONFIG (config)

- **Passed:** 2
- **Failed:** 0
- **Skipped:** 0
- **Success Rate:** 100.0%
- **Duration:** 9.75ms

### TEST (test)

- **Passed:** 11
- **Failed:** 0
- **Skipped:** 0
- **Success Rate:** 100.0%
- **Duration:** 74.78ms

## 🔧 Test Environment

- **OS:** macos
- **Architecture:** aarch64
- **CLI Version:** Not available

### Available Tools

- **dune:** 3.18.2
- **cargo:** cargo 1.87.0 (99624be96 2025-05-06)
- **rustc:** rustc 1.87.0 (17067e9ac 2025-05-09)
- **ocaml:** The OCaml toplevel, version 5.1.1
- **git:** git version 2.49.0

## 📝 Detailed Test Results

### ZK Tests

| Test Name | Status | Duration | Command |
|-----------|--------|----------|----------|
| zk_help | ✅ PASSED | 9.71ms | `causality zk --help` |
| zk_compile_help | ✅ PASSED | 5.19ms | `causality zk compile --help` |
| zk_compile_alias | ✅ PASSED | 5.71ms | `causality zk c --help` |
| zk_compile_privacy_low | ✅ PASSED | 5.92ms | `causality zk compile --input test.ir --output test_low.zk --privacy-level low` |
| zk_compile_privacy_medium | ✅ PASSED | 6.74ms | `causality zk compile --input test.ir --output test_medium.zk --privacy-level medium` |
| zk_compile_privacy_high | ✅ PASSED | 8.22ms | `causality zk compile --input test.ir --output test_high.zk --privacy-level high` |
| zk_compile_privacy_maximum | ✅ PASSED | 10.68ms | `causality zk compile --input test.ir --output test_maximum.zk --privacy-level maximum` |
| zk_compile_proof_system_groth16 | ✅ PASSED | 9.89ms | `causality zk compile --input test.ir --output test_groth16.zk --proof-system groth16` |
| zk_compile_proof_system_plonk | ✅ PASSED | 6.23ms | `causality zk compile --input test.ir --output test_plonk.zk --proof-system plonk` |
| zk_compile_proof_system_stark | ✅ PASSED | 8.72ms | `causality zk compile --input test.ir --output test_stark.zk --proof-system stark` |
| zk_compile_proof_system_marlin | ✅ PASSED | 7.02ms | `causality zk compile --input test.ir --output test_marlin.zk --proof-system marlin` |
| zk_compile_stats | ✅ PASSED | 7.30ms | `causality zk compile --input test.ir --output test_stats.zk --stats` |
| zk_prove_help | ✅ PASSED | 7.86ms | `causality zk prove --help` |
| zk_prove_basic | ✅ PASSED | 6.60ms | `causality zk prove --circuit test.zk --witness witness.json --output proof.zk` |
| zk_verify_help | ✅ PASSED | 5.67ms | `causality zk verify --help` |
| zk_verify_basic | ✅ PASSED | 6.01ms | `causality zk verify --circuit test.zk --proof proof.zk` |
| zk_verify_with_inputs | ✅ PASSED | 7.25ms | `causality zk verify --circuit test.zk --proof proof.zk --public-inputs public_inputs.json` |
| zk_verify_mock | ✅ PASSED | 7.66ms | `causality zk verify --circuit test.zk --proof proof.zk --mock` |
| zk_setup_help | ✅ PASSED | 7.64ms | `causality zk setup --help` |
| zk_setup_basic | ✅ PASSED | 7.28ms | `causality zk setup --circuit test.zk --output-dir setup_output` |
| zk_setup_multi_participants | ✅ PASSED | 7.39ms | `causality zk setup --circuit test.zk --output-dir setup_multi --participants 3` |

### REPL Tests

| Test Name | Status | Duration | Command |
|-----------|--------|----------|----------|
| repl_basic_help | ✅ PASSED | 10.99ms | `causality repl --help` |
| repl_debug_help | ✅ PASSED | 4.59ms | `causality repl --debug --help` |
| repl_show_state_help | ✅ PASSED | 7.94ms | `causality repl --show-state --help` |
| repl_max_steps_help | ✅ PASSED | 7.84ms | `causality repl --max-steps 100 --help` |
| repl_load_tutorial_basic | ✅ PASSED | 8.55ms | `causality repl --load-tutorial basic --help` |
| repl_load_tutorial_effects | ✅ PASSED | 4.57ms | `causality repl --load-tutorial effects --help` |
| repl_load_tutorial_zk | ✅ PASSED | 4.53ms | `causality repl --load-tutorial zk --help` |
| repl_load_tutorial_defi | ✅ PASSED | 10.02ms | `causality repl --load-tutorial defi --help` |
| repl_auto_save_help | ✅ PASSED | 6.94ms | `causality repl --auto-save --help` |
| repl_alias | ✅ PASSED | 7.97ms | `causality r --help` |
| repl_invalid_tutorial | ✅ PASSED | 5.62ms | `causality repl --load-tutorial nonexistent --help` |
| repl_combined_options | ✅ PASSED | 5.63ms | `causality repl --debug --show-state --max-steps 50 --help` |

### DEV Tests

| Test Name | Status | Duration | Command |
|-----------|--------|----------|----------|
| dev_help | ✅ PASSED | 6.94ms | `causality dev --help` |
| dev_alias | ✅ PASSED | 5.62ms | `causality d --help` |
| dev_compile_help | ✅ PASSED | 5.30ms | `causality dev compile --help` |
| dev_compile_alias | ✅ PASSED | 6.55ms | `causality dev c --help` |
| dev_compile_intermediate | ⏭️ SKIPPED | 0.00ns | `causality dev compile -i test.lisp -o test.intermediate --format intermediate` |
| dev_compile_bytecode | ⏭️ SKIPPED | 0.00ns | `causality dev compile -i test.lisp -o test.bytecode --format bytecode` |
| dev_compile_native | ⏭️ SKIPPED | 0.00ns | `causality dev compile -i test.lisp -o test.native --format native` |
| dev_compile_wasm | ⏭️ SKIPPED | 0.00ns | `causality dev compile -i test.lisp -o test.wasm --format wasm` |
| dev_compile_js | ⏭️ SKIPPED | 0.00ns | `causality dev compile -i test.lisp -o test.js --format js` |
| dev_compile_optimize | ⏭️ SKIPPED | 0.00ns | `causality dev compile -i test.lisp -o test_opt.ir --optimize` |
| dev_compile_show_stages | ⏭️ SKIPPED | 0.00ns | `causality dev compile -i test.lisp -o test_stages.ir --show-stages` |
| dev_run_help | ✅ PASSED | 6.77ms | `causality dev run --help` |
| dev_run_alias | ✅ PASSED | 7.70ms | `causality dev r --help` |
| dev_run_file | ⏭️ SKIPPED | 0.00ns | `causality dev run -f test.lisp` |
| dev_run_source | ⏭️ SKIPPED | 0.00ns | `causality dev run -s (+ 1 2)` |
| dev_run_trace | ⏭️ SKIPPED | 0.00ns | `causality dev run -f test.lisp --trace` |
| dev_run_max_steps | ⏭️ SKIPPED | 0.00ns | `causality dev run -f test.lisp --max-steps 1000` |
| dev_serve_help | ✅ PASSED | 7.30ms | `causality dev serve --help` |
| dev_serve_port | ✅ PASSED | 8.55ms | `causality dev serve --port 8080 --help` |
| dev_serve_watch | ✅ PASSED | 6.11ms | `causality dev serve --watch --help` |
| dev_serve_open | ✅ PASSED | 5.24ms | `causality dev serve --open --help` |
| dev_fmt_help | ✅ PASSED | 6.33ms | `causality dev fmt --help` |
| dev_fmt_check | ⏭️ SKIPPED | 0.00ns | `causality dev fmt --check` |
| dev_fmt_files | ⏭️ SKIPPED | 0.00ns | `causality dev fmt test.lisp` |

### DEPLOY Tests

| Test Name | Status | Duration | Command |
|-----------|--------|----------|----------|
| deploy_help | ✅ PASSED | 5.28ms | `causality deploy --help --help` |
| deploy_simulate_help | ✅ PASSED | 6.96ms | `causality deploy simulate --help` |
| deploy_submit_help | ✅ PASSED | 7.22ms | `causality deploy submit --help` |
| deploy_report_help | ✅ PASSED | 4.99ms | `causality deploy report --help` |
| deploy_simulate_chains | ✅ PASSED | 6.96ms | `causality deploy simulate --input test.ir --chains ethereum,polygon` |

### ANALYZE Tests

| Test Name | Status | Duration | Command |
|-----------|--------|----------|----------|
| analyze_help | ✅ PASSED | 11.23ms | `causality analyze --help --help` |
| analyze_code_help | ✅ PASSED | 4.57ms | `causality analyze code --help` |
| analyze_resources_help | ✅ PASSED | 6.03ms | `causality analyze resources --help` |
| analyze_effects_help | ✅ PASSED | 7.37ms | `causality analyze effects --help` |
| analyze_security_help | ✅ PASSED | 10.66ms | `causality analyze security --help` |
| analyze_alias | ✅ PASSED | 7.11ms | `causality a --help` |
| analyze_code_basic | ✅ PASSED | 75.76ms | `causality analyze code .` |
| analyze_resources_basic | ✅ PASSED | 6.94ms | `causality analyze resources -f test.lisp` |
| analyze_effects_basic | ✅ PASSED | 8.12ms | `causality analyze effects -f test.lisp` |
| analyze_security_basic | ✅ PASSED | 7.95ms | `causality analyze security .` |

### PROJECT Tests

| Test Name | Status | Duration | Command |
|-----------|--------|----------|----------|
| project_help | ✅ PASSED | 8.14ms | `causality project --help` |
| project_alias | ✅ PASSED | 11.54ms | `causality p --help` |
| project_new_help | ✅ PASSED | 6.29ms | `causality project new --help` |
| project_new_basic | ✅ PASSED | 7.44ms | `causality project new test-basic-project --template basic` |
| project_new_defi | ✅ PASSED | 13.24ms | `causality project new test-defi-project --template defi` |
| project_new_privacy | ✅ PASSED | 8.39ms | `causality project new test-privacy-project --template privacy` |
| project_new_zk | ✅ PASSED | 11.66ms | `causality project new test-zk-project --template zk` |
| project_new_library | ✅ PASSED | 6.41ms | `causality project new test-library-project --template library` |
| project_new_advanced | ✅ PASSED | 8.19ms | `causality project new test-advanced-project --template advanced` |
| project_new_with_git | ✅ PASSED | 8.01ms | `causality project new git-test-project --template basic --git` |
| project_new_with_description | ✅ PASSED | 8.74ms | `causality project new desc-test-project --template basic --description A test project with description` |
| project_init_help | ✅ PASSED | 6.74ms | `causality project init --help` |
| project_init_empty | ✅ PASSED | 7.29ms | `causality project init` |
| project_init_force | ✅ PASSED | 6.94ms | `causality project init --force` |
| project_build_help | ✅ PASSED | 7.35ms | `causality project build --help` |
| project_build_alias | ✅ PASSED | 5.32ms | `causality project b --help` |
| project_build_release | ✅ PASSED | 7.72ms | `causality project build --release --help` |
| project_build_timings | ✅ PASSED | 6.95ms | `causality project build --timings --help` |
| project_clean_help | ✅ PASSED | 6.96ms | `causality project clean --help` |
| project_clean_deep | ✅ PASSED | 7.08ms | `causality project clean --deep --help` |
| project_status_help | ✅ PASSED | 5.46ms | `causality project status --help` |
| project_status_alias | ✅ PASSED | 6.27ms | `causality project s --help` |
| project_status_deps | ✅ PASSED | 5.49ms | `causality project status --deps --help` |
| project_add_help | ✅ PASSED | 5.54ms | `causality project add --help` |
| project_add_with_version | ✅ PASSED | 5.35ms | `causality project add test-package --version 1.0.0 --help` |

### HELP Tests

| Test Name | Status | Duration | Command |
|-----------|--------|----------|----------|
| help_basic | ✅ PASSED | 8.76ms | `causality help` |
| help_short_flag | ✅ PASSED | 7.60ms | `causality -h` |
| help_long_flag | ✅ PASSED | 9.12ms | `causality --help` |
| help_tutorial | ⏭️ SKIPPED | 0.00ns | `causality help tutorial` |
| help_guides | ⏭️ SKIPPED | 0.00ns | `causality help guides` |
| help_reference | ⏭️ SKIPPED | 0.00ns | `causality help reference` |
| help_examples | ⏭️ SKIPPED | 0.00ns | `causality help examples` |
| help_api | ⏭️ SKIPPED | 0.00ns | `causality help api` |
| help_troubleshooting | ⏭️ SKIPPED | 0.00ns | `causality help troubleshooting` |
| help_repl_command | ✅ PASSED | 6.34ms | `causality repl --help` |
| help_project_command | ✅ PASSED | 7.06ms | `causality project --help` |
| help_dev_command | ✅ PASSED | 6.39ms | `causality dev --help` |
| help_zk_command | ✅ PASSED | 9.07ms | `causality zk --help` |
| help_deploy_command | ✅ PASSED | 4.78ms | `causality deploy --help` |
| help_analyze_command | ✅ PASSED | 8.07ms | `causality analyze --help` |
| help_test_command | ✅ PASSED | 6.42ms | `causality test --help` |
| help_inspect_command | ✅ PASSED | 7.08ms | `causality inspect --help` |
| help_viz_command | ✅ PASSED | 8.13ms | `causality viz --help` |
| help_config_command | ✅ PASSED | 11.13ms | `causality config --help` |
| help_project_new | ✅ PASSED | 9.13ms | `causality project new --help` |
| help_project_build | ✅ PASSED | 10.25ms | `causality project build --help` |
| help_project_status | ✅ PASSED | 5.41ms | `causality project status --help` |
| help_dev_compile | ✅ PASSED | 6.90ms | `causality dev compile --help` |
| help_dev_run | ✅ PASSED | 12.13ms | `causality dev run --help` |
| help_dev_serve | ✅ PASSED | 6.39ms | `causality dev serve --help` |
| help_zk_compile | ✅ PASSED | 5.57ms | `causality zk compile --help` |
| help_zk_prove | ✅ PASSED | 6.24ms | `causality zk prove --help` |
| help_zk_verify | ✅ PASSED | 4.89ms | `causality zk verify --help` |
| help_deploy_simulate | ✅ PASSED | 6.23ms | `causality deploy simulate --help` |
| help_deploy_submit | ✅ PASSED | 4.57ms | `causality deploy submit --help` |
| help_analyze_code | ✅ PASSED | 8.39ms | `causality analyze code --help` |
| help_analyze_resources | ✅ PASSED | 5.29ms | `causality analyze resources --help` |
| help_test_unit | ✅ PASSED | 14.30ms | `causality test unit --help` |
| help_test_e2e | ✅ PASSED | 5.66ms | `causality test e2e --help` |
| help_invalid_topic | ✅ PASSED | 6.43ms | `causality help nonexistent` |

### INSPECT Tests

| Test Name | Status | Duration | Command |
|-----------|--------|----------|----------|
| inspect_help | ✅ PASSED | 6.70ms | `causality inspect --help` |
| inspect_alias | ✅ PASSED | 4.48ms | `causality i --help` |

### VIZ Tests

| Test Name | Status | Duration | Command |
|-----------|--------|----------|----------|
| viz_help | ✅ PASSED | 6.30ms | `causality viz --help` |
| viz_alias | ✅ PASSED | 7.31ms | `causality v --help` |

### CONFIG Tests

| Test Name | Status | Duration | Command |
|-----------|--------|----------|----------|
| config_help | ✅ PASSED | 4.56ms | `causality config --help` |
| config_alias | ✅ PASSED | 5.14ms | `causality c --help` |

### TEST Tests

| Test Name | Status | Duration | Command |
|-----------|--------|----------|----------|
| test_help | ✅ PASSED | 6.07ms | `causality test --help` |
| test_alias | ✅ PASSED | 6.39ms | `causality t --help` |
| test_unit_help | ✅ PASSED | 10.75ms | `causality test unit --help` |
| test_effects_help | ✅ PASSED | 6.36ms | `causality test effects --help` |
| test_integration_help | ✅ PASSED | 5.45ms | `causality test integration --help` |
| test_e2e_help | ✅ PASSED | 6.61ms | `causality test e2e --help` |
| test_integration_alias | ✅ PASSED | 5.13ms | `causality test int --help` |
| test_unit_basic | ✅ PASSED | 6.65ms | `causality test unit` |
| test_unit_coverage | ✅ PASSED | 5.76ms | `causality test unit --coverage` |
| test_effects_property | ✅ PASSED | 6.84ms | `causality test effects --property-based` |
| test_e2e_chains | ✅ PASSED | 8.58ms | `causality test e2e --chains ethereum,polygon` |

