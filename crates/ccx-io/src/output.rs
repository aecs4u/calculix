use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use ccx_model::ModelSummary;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    Success,
    Failed,
}

impl JobStatus {
    fn as_str(self) -> &'static str {
        match self {
            JobStatus::Success => "SUCCESS",
            JobStatus::Failed => "FAILED",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JobReport {
    pub job_name: String,
    pub analysis_type: String,
    pub num_nodes: usize,
    pub num_elements: usize,
    pub num_dofs: usize,
    pub num_equations: usize,
    pub status: JobStatus,
    pub message: String,
}

impl JobReport {
    pub fn from_summary(
        job_name: impl Into<String>,
        analysis_type: impl Into<String>,
        summary: &ModelSummary,
        status: JobStatus,
        message: impl Into<String>,
    ) -> Self {
        let num_nodes = summary.node_rows;
        let num_elements = summary.element_rows;
        let num_dofs = num_nodes * 3;
        Self {
            job_name: job_name.into(),
            analysis_type: analysis_type.into(),
            num_nodes,
            num_elements,
            num_dofs,
            num_equations: num_dofs,
            status,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputBundle {
    pub dat_path: PathBuf,
    pub sta_path: PathBuf,
    pub frd_path: PathBuf,
}

pub fn write_output_bundle(dir: impl AsRef<Path>, report: &JobReport) -> io::Result<OutputBundle> {
    let dir = dir.as_ref();
    fs::create_dir_all(dir)?;

    let dat_path = dir.join(format!("{}.dat", report.job_name));
    let sta_path = dir.join(format!("{}.sta", report.job_name));
    let frd_path = dir.join(format!("{}.frd", report.job_name));

    write_dat(&dat_path, report)?;
    write_sta(&sta_path, report)?;
    write_frd_stub(&frd_path, report)?;

    Ok(OutputBundle {
        dat_path,
        sta_path,
        frd_path,
    })
}

pub fn write_dat(path: impl AsRef<Path>, report: &JobReport) -> io::Result<()> {
    let path = path.as_ref();
    ensure_parent_dir(path)?;
    let body = format!(
        "*CCX DAT REPORT\n\
         JOB: {}\n\
         ANALYSIS: {}\n\
         STATUS: {}\n\
         NODES: {}\n\
         ELEMENTS: {}\n\
         DOFS: {}\n\
         EQUATIONS: {}\n\
         MESSAGE: {}\n",
        report.job_name,
        report.analysis_type,
        report.status.as_str(),
        report.num_nodes,
        report.num_elements,
        report.num_dofs,
        report.num_equations,
        report.message
    );
    fs::write(path, body)
}

pub fn write_sta(path: impl AsRef<Path>, report: &JobReport) -> io::Result<()> {
    let path = path.as_ref();
    ensure_parent_dir(path)?;
    let status_code = match report.status {
        JobStatus::Success => 0,
        JobStatus::Failed => 1,
    };
    let body = format!(
        "*CCX STA REPORT\n\
         STEP  INC   STATUS  DOFS  EQS\n\
         1     1     {}      {}    {}\n\
         # {}\n",
        status_code, report.num_dofs, report.num_equations, report.message
    );
    fs::write(path, body)
}

pub fn write_frd_stub(path: impl AsRef<Path>, report: &JobReport) -> io::Result<()> {
    let path = path.as_ref();
    ensure_parent_dir(path)?;
    let body = format!(
        "    1PSTEP      1PINC      1\n\
         -1\n\
         100C{}\n\
         9999\n\
         # STATUS={}\n\
         # ANALYSIS={}\n\
         # NODES={} ELEMENTS={}\n",
        report.job_name,
        report.status.as_str(),
        report.analysis_type,
        report.num_nodes,
        report.num_elements
    );
    fs::write(path, body)
}

fn ensure_parent_dir(path: &Path) -> io::Result<()> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn writes_dat_sta_and_frd_bundle() {
        let root = unique_temp_dir("ccx_io_bundle");
        let report = JobReport {
            job_name: "beam_static".to_string(),
            analysis_type: "LinearStatic".to_string(),
            num_nodes: 12,
            num_elements: 6,
            num_dofs: 36,
            num_equations: 36,
            status: JobStatus::Success,
            message: "Run completed".to_string(),
        };

        let out = write_output_bundle(&root, &report).expect("output bundle should write");
        assert!(out.dat_path.exists());
        assert!(out.sta_path.exists());
        assert!(out.frd_path.exists());

        let dat = fs::read_to_string(&out.dat_path).expect("dat should be readable");
        assert!(dat.contains("JOB: beam_static"));
        assert!(dat.contains("STATUS: SUCCESS"));
    }

    #[test]
    fn writes_failed_status_in_sta() {
        let root = unique_temp_dir("ccx_io_sta");
        let sta = root.join("job.sta");
        let report = JobReport {
            job_name: "job".to_string(),
            analysis_type: "Dynamic".to_string(),
            num_nodes: 4,
            num_elements: 2,
            num_dofs: 12,
            num_equations: 12,
            status: JobStatus::Failed,
            message: "Diverged".to_string(),
        };
        write_sta(&sta, &report).expect("sta should write");
        let content = fs::read_to_string(&sta).expect("sta should be readable");
        assert!(content.contains(" 1 "));
        assert!(content.contains("Diverged"));
    }

    fn unique_temp_dir(prefix: &str) -> PathBuf {
        let pid = std::process::id();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be valid")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}_{pid}_{nanos}"))
    }
}
