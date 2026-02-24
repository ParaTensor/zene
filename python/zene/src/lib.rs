use pyo3::prelude::*;
use ::zene::agent::engine::ZeneEngine;
use ::zene::engine::contracts::{RunRequest, AgentEvent};
use ::zene::config::AgentConfig;
use ::zene::engine::session::store::{FileSessionStore, InMemorySessionStore, SessionStore};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::PathBuf;

#[pyclass]
struct ZeneClient {
    engine: Arc<Mutex<Option<ZeneEngine>>>,
    config: AgentConfig,
}

#[pymethods]
impl ZeneClient {
    #[new]
    #[pyo3(signature = (
        planner_provider=None, planner_model=None, planner_api_key=None,
        executor_provider=None, executor_model=None, executor_api_key=None,
        reflector_provider=None, reflector_model=None, reflector_api_key=None
    ))]
    fn new(
        planner_provider: Option<String>,
        planner_model: Option<String>,
        planner_api_key: Option<String>,
        executor_provider: Option<String>,
        executor_model: Option<String>,
        executor_api_key: Option<String>,
        reflector_provider: Option<String>,
        reflector_model: Option<String>,
        reflector_api_key: Option<String>,
    ) -> PyResult<Self> {
        let mut config = AgentConfig::default();

        // Update config based on arguments
        if let Some(p) = planner_provider { config.planner.provider = p; }
        if let Some(m) = planner_model { config.planner.model = m; }
        if let Some(k) = planner_api_key { config.planner.api_key = k; }

        if let Some(p) = executor_provider { config.executor.provider = p; }
        if let Some(m) = executor_model { config.executor.model = m; }
        if let Some(k) = executor_api_key { config.executor.api_key = k; }

        if let Some(p) = reflector_provider { config.reflector.provider = p; }
        if let Some(m) = reflector_model { config.reflector.model = m; }
        if let Some(k) = reflector_api_key { config.reflector.api_key = k; }

        Ok(ZeneClient {
            engine: Arc::new(Mutex::new(None)),
            config,
        })
    }

    fn init(&self, work_dir: Option<String>) -> PyResult<()> {
        let config = self.config.clone();
        let engine_arc = self.engine.clone();
        
        // Use tokio runtime to initialize async engine
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let session_store: Arc<dyn SessionStore> = if let Some(dir) = work_dir {
                Arc::new(FileSessionStore::new(PathBuf::from(dir)).map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to create FileSessionStore: {}", e))
                })?)
            } else {
                Arc::new(InMemorySessionStore::new())
            };

            let engine = ZeneEngine::new(config, session_store).await.map_err(|e| {
                pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to init engine: {}", e))
            })?;
            let mut guard = engine_arc.lock().await;
            *guard = Some(engine);
            Ok(())
        })
    }

    fn run(&self, prompt: String, session_id: String) -> PyResult<Vec<HashMap<String, String>>> {
        let engine_arc = self.engine.clone();
        
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let mut guard = engine_arc.lock().await;
            if let Some(engine) = guard.as_mut() {
                let request = RunRequest {
                    prompt,
                    session_id,
                    env_vars: None,
                };
                
                let mut events = Vec::new();
                
                let mut rx = engine.run_stream(request).await.map_err(|e| {
                    pyo3::exceptions::PyRuntimeError::new_err(format!("Failed to run task: {}", e))
                })?;

                while let Some(event) = rx.recv().await {
                    let mut map = HashMap::new();
                    // Basic serialization of AgentEvent to Dict
                    match event {
                        AgentEvent::PlanningStarted => {
                            map.insert("type".to_string(), "PlanningStarted".to_string());
                        }
                        AgentEvent::TaskStarted { id, description } => {
                            map.insert("type".to_string(), "TaskStarted".to_string());
                            map.insert("id".to_string(), id.to_string());
                            map.insert("description".to_string(), description);
                        }
                        AgentEvent::ThoughtDelta(delta) => {
                            map.insert("type".to_string(), "ThoughtDelta".to_string());
                            map.insert("content".to_string(), delta);
                        }
                        AgentEvent::Finished(output) => {
                            map.insert("type".to_string(), "Finished".to_string());
                            map.insert("content".to_string(), output);
                        }
                        AgentEvent::Error { code, message } => {
                            map.insert("type".to_string(), "Error".to_string());
                            map.insert("code".to_string(), code);
                            map.insert("message".to_string(), message);
                        }
                        // Add more events as needed
                        _ => {
                            map.insert("type".to_string(), format!("{:?}", event));
                        }
                    }
                    events.push(map);
                }
                
                Ok(events)
            } else {
                Err(pyo3::exceptions::PyRuntimeError::new_err("Engine not initialized. Call init() first."))
            }
        })
    }
}

/// A Python module implemented in Rust.
#[pymodule]
fn zene(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<ZeneClient>()?;
    Ok(())
}
