extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;

#[derive(Parser)]
#[grammar = "pipeline.pest"]
pub struct PipelineParser;

pub struct Jenkinsfile {
    pub name: String,
    pub stages: Vec<JenkinsStage>,
    pub post: Vec<PostConfig>,
}

impl Default for Jenkinsfile {
    fn default() -> Self {
        Jenkinsfile {
            name: "".to_string(),
            stages: vec![],
            post: vec![]
        }
    }
}

impl Jenkinsfile {
    pub fn from_str(str: &str) -> Option<Jenkinsfile> {
        let mut _parser = PipelineParser::parse(Rule::pipeline, str).unwrap_or_else(|e| panic!("{}", e));

        return Some(Jenkinsfile::default());
    }
}

pub struct JenkinsStage {
    pub name: String,
    pub jobs: Vec<JenkinsJob>,
}

pub struct JenkinsJob {
    pub name: String,
    pub job: String,
}

pub struct PostConfig {
    pub key: String,
    pub value: Vec<JenkinsJob>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn should_parse_hello_world() {
        let code = r#"pipeline {
    agent { docker 'maven:3.3.3' }
    stages {
        stage('build') {
            steps {
                sh 'mvn --version'
            }
        }
    }
}
        "#;
        Jenkinsfile::from_str(code);
    }
}