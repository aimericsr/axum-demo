/*!
  An application to start a web server in rust using the axum framework. It is fully observable
  via the exposition of the /metrics endpoint and the export of all the traces to a otel-collector

# Overview

This section gives a brief overview of the primary types in this crate:

* [`config`] load the environnement variables
* [`init_subscriber`](observability::tracing::init_subscriber) is used to setup the instrumentation of the application
* [`Application`](startup::Application) is the primary type and represents the application information
and how to lunch it.

# Basic Usage
Build the app and lunch it
```
use axum_demo::config::config;
use axum_demo::observability::tracing::init_subscriber;
use axum_demo::startup::Application;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let config = config();

    init_subscriber();

    let _ = Application::new(config);
    Application::run_until_stopped(config).await;
    Ok(())
}
```
*/

/// For dev only, shoud be remove in a future release
mod _dev_utils;
/// Load the config for the app, can come from differents locations
pub mod config;
/// Handle all the JWT related operations
mod crypt;
/// Excract the context from the HTTP request
mod ctx;
/// All possible errors that can occurs
mod error;
/// All the model layer related functionnality : modele, controller ...
mod model;
/// Centralize the observability capabilities of the application : tracing and metrics
pub mod observability;
/// All of the functions needed to start the application
pub mod startup;
/// All the routing and controllers logic
mod web;
