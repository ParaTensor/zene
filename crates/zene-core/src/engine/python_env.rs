use anyhow::Result;
use std::path::PathBuf;
use tokio::process::Command;
use tracing::info;

pub struct PythonEnv {
    root: PathBuf,
}

impl PythonEnv {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn get_venv_python(&self) -> PathBuf {
        self.root.join(".venv").join("bin").join("python")
    }

    pub fn get_venv_pip(&self) -> PathBuf {
        self.root.join(".venv").join("bin").join("pip")
    }

    pub async fn ensure_venv(&self) -> Result<PathBuf> {
        let venv_path = self.root.join(".venv");
        if venv_path.exists() {
            return Ok(self.get_venv_python());
        }

        info!("Creating virtual environment at {:?}", venv_path);

        if Command::new("uv").arg("--version").output().await.is_ok() {
            info!("Using uv to create venv");
            Command::new("uv")
                .arg("venv")
                .arg(".venv")
                .current_dir(&self.root)
                .output()
                .await?;
        } else {
            info!("Using python3 to create venv");
            Command::new("python3")
                .arg("-m")
                .arg("venv")
                .arg(".venv")
                .current_dir(&self.root)
                .output()
                .await?;
        }

        Ok(self.get_venv_python())
    }

    pub async fn install_requirements(&self) -> Result<()> {
        let req_path = self.root.join("requirements.txt");
        if !req_path.exists() {
            return Ok(());
        }

        if Command::new("uv").arg("--version").output().await.is_ok() {
            info!("Installing requirements using uv");
             Command::new("uv")
                .arg("pip")
                .arg("install")
                .arg("-r")
                .arg("requirements.txt")
                .env("VIRTUAL_ENV", self.root.join(".venv"))
                .current_dir(&self.root)
                .output()
                .await?;
        } else {
            info!("Installing requirements using pip");
            // Standard pip install in venv
            let pip_path = self.get_venv_pip();
            Command::new(pip_path)
                .arg("install")
                .arg("-r")
                .arg("requirements.txt")
                .current_dir(&self.root)
                .output()
                .await?;
        }

        Ok(())
    }
}
