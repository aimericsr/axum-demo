/*!
  An application to start a web server in rust using the axum framework. It is fully observable
  via the exposition of the /metrics endpoint and the export of all the traces to an otel collector.

# Overview

This section gives a brief overview of the primary types in this crate:

* [`config`] load the environnement variables
* [`init_subscriber`](observability::tracing::init_subscriber) is used to setup the instrumentation of the application
* [`Application`](startup::Application) is the primary type and represents the application information
and how to lunch it.

# Basic Usage
Build the app and lunch it
```rust,no_run
use axum_demo::config::get_configuration;
use axum_demo::observability::tracing::init_subscriber;
use axum_demo::startup::Application;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Retreive the configuration
    let config = get_configuration().expect("Failed to read configuration.");

    // Init the tracing
    init_subscriber(&config.otel);

    // Build the app
    let application = Application::build(config)
        .await
        .expect("Failed to build the app");

    // Lunch the application to start listening to requests
    application
        .run_until_stopped()
        .await
        .expect("Failed to lunch the app");
    Ok(())
}
```
*/

/// For dev only, shoud be remove in a future release
mod _dev_utils;
/// Load the config for the app, can come from differents locations
pub mod config;
/// Handle all the JWT related operations
pub mod crypt;
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
pub mod web;
