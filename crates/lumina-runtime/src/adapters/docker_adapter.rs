use crate::value::Value;
use std::collections::HashMap;
use bollard::Docker;
use bollard::container::{Config, CreateContainerOptions, ListContainersOptions, RemoveContainerOptions};
use bollard::models::HostConfig;
use bollard::secret::PortBinding;
use bollard::image::CreateImageOptions;
use futures_util::stream::StreamExt;
use tokio::runtime::Handle;
use tokio::task::block_in_place;

/// A native Lumina v2.1 adapter for Docker infrastructure.
/// Communicates directly with the Docker Daemon via Unix Socket using bollard.
pub struct DockerAdapter {
    entity_name: String,
    /// Cache of known container states to prevent redundant API calls
    known_states: HashMap<String, String>,
    docker: Docker,
}

impl DockerAdapter {
    pub fn new(entity_name: &str) -> Self {
        // Fallback to local defaults (Unix socket on Linux/macOS, Named pipe on Windows)
        let docker = Docker::connect_with_local_defaults().expect("Failed to connect to Docker daemon");
        Self {
            entity_name: entity_name.to_string(),
            known_states: HashMap::new(),
            docker,
        }
    }
}

impl crate::adapter::LuminaAdapter for DockerAdapter {
    fn entity_name(&self) -> &str {
        &self.entity_name
    }

    fn poll(&mut self) -> Vec<(String, String, Value)> {
        let mut updates = vec![];
        
        let mut options = ListContainersOptions::<String>::default();
        options.all = true;

        let result = block_in_place(|| {
            Handle::current().block_on(self.docker.list_containers(Some(options)))
        });

        if let Ok(containers) = result {
            for container in containers {
                let name = container.names.unwrap_or_default()
                    .first()
                    .map(|n| n.trim_start_matches('/').to_string())
                    .unwrap_or_default();
                
                if name.is_empty() { continue; }

                let state = container.state.unwrap_or_default();
                let status = match state.as_str() {
                    "running" => "running",
                    "exited" | "dead" => "stopped",
                    "created" | "restarting" => "starting",
                    _ => "down",
                };

                self.known_states.insert(name.clone(), status.to_string());
                updates.push((name.clone(), "status".to_string(), Value::Text(status.to_string())));
                updates.push((name, "verified".to_string(), Value::Bool(true)));
            }
        }
        
        updates
    }

    fn on_write(&mut self, instance: &str, field: &str, value: &Value) {
        if field == "status" {
            if let Some(status) = value.as_text() {
                let current_known = self.known_states.get(instance).map(|s| s.as_str());
                
                if Some(status) != current_known {
                    if status == "running" {
                        let _ = block_in_place(|| Handle::current().block_on(self.docker.start_container::<String>(instance, None)));
                        self.known_states.insert(instance.to_string(), "running".to_string());
                    } else if status == "stopped" {
                        let _ = block_in_place(|| Handle::current().block_on(self.docker.stop_container(instance, None)));
                        self.known_states.insert(instance.to_string(), "stopped".to_string());
                    }
                }
            }
        }
    }

    fn provision(&mut self, instance: &str, desired: &HashMap<String, Value>) -> Result<(), String> {
        let status = desired.get("status").and_then(|v| v.as_text()).unwrap_or("running");
        let image = desired.get("image").and_then(|v| v.as_text()).unwrap_or("nginx:alpine");
        let port = desired.get("port").and_then(|v| v.as_number());

        if status == "stopped" {
            let _ = block_in_place(|| Handle::current().block_on(self.docker.stop_container(instance, None)));
            return Ok(());
        }

        // Check if exists
        let exists = block_in_place(|| Handle::current().block_on(self.docker.inspect_container(instance, None))).is_ok();
        
        if !exists {
            // Pull the image first (Bollard requires this unlike CLI `docker run`)
            println!("\x1b[33m[PULLING] Image '{}' for '{}'...\x1b[0m", image, instance);
            let _ = block_in_place(|| {
                Handle::current().block_on(async {
                    let mut stream = self.docker.create_image(Some(CreateImageOptions {
                        from_image: image,
                        ..Default::default()
                    }), None, None);
                    while let Some(_) = stream.next().await {}
                    Ok::<(), bollard::errors::Error>(())
                })
            });

            let mut port_bindings = HashMap::new();
            if let Some(p) = port {
                let p_int = p as u32;
                if p_int > 0 {
                    let target_p = desired.get("target_port").and_then(|v| v.as_number()).unwrap_or(80.0) as u32;
                    let target_port_str = format!("{}/tcp", target_p);
                    port_bindings.insert(
                        target_port_str,
                        Some(vec![PortBinding {
                            host_ip: Some("0.0.0.0".into()),
                            host_port: Some(p_int.to_string()),
                        }]),
                    );
                }
            }

            let mut env = vec![];
            if let Some(Value::Text(env_str)) = desired.get("env_vars") {
                if env_str != "NONE" && !env_str.is_empty() {
                    for var in env_str.split(',') {
                        env.push(var.trim().to_string());
                    }
                }
            }

            let host_config = HostConfig {
                port_bindings: Some(port_bindings),
                ..Default::default()
            };

            let config = Config {
                image: Some(image.to_string()),
                env: Some(env),
                host_config: Some(host_config),
                ..Default::default()
            };

            let options = Some(CreateContainerOptions {
                name: instance,
                platform: None,
            });

            let create_result = block_in_place(|| Handle::current().block_on(self.docker.create_container(options, config)));
            match create_result {
                Ok(_) => {
                    let start_result = block_in_place(|| Handle::current().block_on(self.docker.start_container::<String>(instance, None)));
                    if let Err(e) = start_result {
                        eprintln!("\x1b[31m[DOCKER ERROR] Failed to start {}: {}\x1b[0m", instance, e);
                        return Err(e.to_string());
                    }
                    if let Some(p) = port {
                        let p_int = p as u32;
                        if p_int > 0 {
                            println!("\x1b[32m[CREATED] Container '{}' is live on port {} → http://localhost:{}\x1b[0m", instance, p_int, p_int);
                        } else {
                            println!("\x1b[32m[CREATED] Container '{}' is live (internal)\x1b[0m", instance);
                        }
                    } else {
                        println!("\x1b[32m[CREATED] Container '{}' is live (internal)\x1b[0m", instance);
                    }
                }
                Err(e) => {
                    eprintln!("\x1b[31m[DOCKER ERROR] Failed to create {}: {}\x1b[0m", instance, e);
                    return Err(e.to_string());
                }
            }
        } else {
            // Existence check
            println!("\x1b[34m[VERIFIED] Resource '{}' already exists. Synchronizing state...\x1b[0m", instance);
            let _ = block_in_place(|| Handle::current().block_on(self.docker.start_container::<String>(instance, None)));
        }
        
        Ok(())
    }

    fn reconcile(&mut self, instance: &str, desired: &HashMap<String, Value>) -> Result<(), String> {
        self.provision(instance, desired)
    }

    fn destroy(&mut self, instance: &str) -> Result<(), String> {
        let options = Some(RemoveContainerOptions {
            force: true,
            ..Default::default()
        });
        let remove_result = block_in_place(|| Handle::current().block_on(self.docker.remove_container(instance, options)));
        match remove_result {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }
}
