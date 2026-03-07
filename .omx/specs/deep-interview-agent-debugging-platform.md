# Execution Spec: Agent Debugging Platform

## Goal

Build a Blinc-native agent-debugging platform that lets a Rust-native automation client drive desktop and headless Blinc apps, capture a full trace of commands and resulting runtime evidence, and inspect failures in a forensic debugger.

## Confirmed Constraints

- Desktop and headless are required in MVP.
- Mobile is out of scope in MVP.
- Rust-native API is the primary automation surface.
- Scenario DSL and state-machine playbooks are both required.
- Stable ID and semantic locators are both required.
- `probar` is a reference, not a hard dependency.

## Required Outputs

- shared trace schema
- automation command/session crate
- playbook compiler crate
- recorder integration into the trace model
- desktop and headless execution paths
- debugger UI upgraded for trace forensics

## Success Criteria

- same scenario can run in desktop and headless modes
- both locator families work in MVP
- both playbook formats compile into one execution plan
- trace captures commands, runtime events, locator resolution, state snapshots, render evidence, and assertions
- debugger can investigate failed traces without rerunning the app

