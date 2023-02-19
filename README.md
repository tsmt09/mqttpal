# MQTT Pal

## Description 
The goal of this project is to deliver a general MQTT GUI whose internal code is so far abstracted that it can be delivered into different UI variants.

Currently, the following features are planned:
- [] UI
  - Server Mode with Web UI
  - Client Mode (WASM only) with Web UI, also delivered as Tauri App
- [] MQTT Monitoring
  - subscribe to Topics
  - collect statistics
  - store statistics
- [] Automation (for now only available in server)
  - simple automation tasks which can do things based on a topic input
  - scriptable with RHAI / Lua / Python

One of the main features is that all the different UIs are using the same (yaml) config files, so they are interchangeable between your local UI, server UI or Terminal UI.

## Motivation

Personally, I think the MQTT world is missing a sophisticated UI which has enough flexibility to be deployed in different locations, as a local Desktop Application, as a Web UI or Terminal user interface. Configuration files for target MQTT servers and subscribed topics should be exchangeable.

I think, that Rust is giving the perfect opportunity to reach this goal due to the very detailed workspace structure, available libraries and code exporting possibilites (C lib, WASM, Python etc.).

Also, it's just a fun project for me to get deeper insight into Cargo Workspaces, Tauri and Web Backends such as Rocket or Warp.
  
