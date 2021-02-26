#[macro_use]
extern crate lazy_static;
extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::error::Error as PestError;
use pest::error::ErrorVariant;
use pest::iterators::Pairs;
use pest::Parser;
use regex::Regex;

#[derive(Parser)]
#[grammar = "pipeline.pest"]
pub struct PipelineParser;

#[derive(Debug, PartialEq, Eq, Clone)]
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
            post: vec![],
        }
    }
}

impl Jenkinsfile {
    pub fn from_str(str: &str) -> Option<Jenkinsfile> {
        let mut jenkinsfile = Jenkinsfile::default();

        let _result = jenkinsfile.parse_pipeline_string(str);

        return Some(jenkinsfile);
    }

    pub fn parse_pipeline_string(&mut self, buffer: &str) -> Result<(), PestError<Rule>> {
        if !self.is_declarative(buffer) {
            return Err(PestError::new_from_pos(
                ErrorVariant::CustomError {
                    message: "The buffer does not appear to be a Declarative Pipeline, I couldn't find pipeline { }".to_string(),
                },
                pest::Position::from_start(buffer),
            ));
        }

        let mut parser = PipelineParser::parse(Rule::pipeline, buffer)?;

        while let Some(parsed) = parser.next() {
            match parsed.as_rule() {
                Rule::stagesDecl => {
                    self.stages = self.parse_stages(&mut parsed.into_inner());
                }
                _ => {}
            }
        }

        Ok(())
    }

    /**
     * Run a quick sanity check to determine whether the given buffer appears to
     * be a Declarative Pipeline or not.
     */
    fn is_declarative(&mut self, buffer: &str) -> bool {
        lazy_static! {
        static ref RE: Regex = Regex::new(r"pipeline(\s+)?\{").expect("Failed to make regex");
    }
        RE.is_match(buffer)
    }

    /**
     * Make sure that the stage has the required directives, otherwise throw
     * out a CustomError
     */
    fn parse_stage(&mut self, parser: &mut Pairs<Rule>) -> JenkinsStage {
        let mut current_stage = JenkinsStage::default();
        while let Some(parsed) = parser.next() {
            match parsed.as_rule() {
                Rule::string => {
                    current_stage.name = self.parse_string(&mut parsed.into_inner());
                }
                Rule::stepsDecl => {
                    current_stage.steps = self.parse_steps(&mut parsed.into_inner());
                }
                Rule::parallelDecl => {
                    current_stage.is_parallel = true;
                    let mut parallel_decl = parsed.into_inner();
                    while let Some(parallel_parsed) = parallel_decl.next() {
                        match parallel_parsed.as_rule() {
                            Rule::stage => {
                                let stage = self.parse_stage(&mut parallel_parsed.into_inner());
                                current_stage.sub_stages.push(stage);
                            }
                            _ => {}
                        }
                    }
                }
                Rule::stagesDecl => {
                    current_stage.sub_stages = self.parse_stages(&mut parsed.into_inner());
                }
                _ => {}
            }
        }

        return current_stage
    }

    fn parse_string(&mut self, parser: &mut Pairs<Rule>) -> String {
        while let Some(parsed) = parser.next() {
            match parsed.as_rule() {
                Rule::triple_single_quoted
                | Rule::single_quoted
                | Rule::triple_double_quoted
                | Rule::double_quoted => {
                    for field in parsed.into_inner() {
                        match field.as_rule() {
                            Rule::inner_single_str
                            | Rule::inner_triple_single_str
                            | Rule::inner_triple_double_str
                            | Rule::inner_double_str => {
                                return field.as_str().to_string();
                            }
                            _ => {}
                        }
                    }
                }
                _ => {
                    println!("not support {:?}", parsed.as_rule());
                }
            }
        }

        return "".to_string();
    }
    fn parse_stages(&mut self, parser: &mut Pairs<Rule>) -> Vec<JenkinsStage> {
        let mut stages: Vec<JenkinsStage> = vec![];
        while let Some(parsed) = parser.next() {
            match parsed.as_rule() {
                Rule::stage => {
                    stages.push(self.parse_stage(&mut parsed.into_inner()));
                }
                _ => {}
            }
        }


        stages
    }

    fn parse_steps(&mut self, parser: &mut Pairs<Rule>) -> Vec<String> {
        let mut steps: Vec<String> = vec![];
        while let Some(parsed) = parser.next() {
            match parsed.as_rule() {
                Rule::step => {
                    steps.push(parsed.as_str().to_string());
                }
                Rule::scriptStep => {
                    steps.push(parsed.as_str().to_string());
                }
                _ => {}
            }
        }

        steps
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct JenkinsStage {
    pub name: String,
    pub steps: Vec<String>,
    pub is_parallel: bool,
    pub sub_stages: Vec<JenkinsStage>,
}

impl Default for JenkinsStage {
    fn default() -> Self {
        JenkinsStage {
            name: "".to_string(),
            steps: vec![],
            is_parallel: false,
            sub_stages: vec![],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PostConfig {
    pub key: String,
    pub value: Vec<String>,
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
        let jenkinsfile = Jenkinsfile::from_str(code).unwrap();
        assert_eq!(1, jenkinsfile.stages.len());
        assert_eq!("build", jenkinsfile.stages[0].name);
        println!("{:?}", jenkinsfile.stages[0].steps);
        assert_eq!(1, jenkinsfile.stages[0].steps.len());
    }

    #[test]
    pub fn should_parse_echo() {
        let code = r#"pipeline {
    stages {
        stage('build') {
            steps {
                sleep 10
                sh 'mvn --version'
            }
        }
    }
}
        "#;
        let jenkinsfile = Jenkinsfile::from_str(code).unwrap();
        assert_eq!(1, jenkinsfile.stages.len());
        assert_eq!(2, jenkinsfile.stages[0].steps.len());
    }

    #[test]
    pub fn should_parse_sub_stages() {
        let code = r#"pipeline {
    agent none
    stages {
        stage('Sequential') {
            agent {
                label 'for-sequential'
            }
            environment {
                FOR_SEQUENTIAL = "some-value"
            }
            stages {
                stage('In Sequential 1') {
                    steps {
                        echo "In Sequential 1"
                    }
                }
                stage('In Sequential 2') {
                    steps {
                        echo "In Sequential 2"
                    }
                }
            }
        }
    }
}
        "#;
        let jenkinsfile = Jenkinsfile::from_str(code).unwrap();

        assert_eq!(1, jenkinsfile.stages.len());
        assert_eq!(2, jenkinsfile.stages[0].sub_stages.len());
        assert_eq!(1, jenkinsfile.stages[0].sub_stages[0].steps.len());
    }

    #[test]
    pub fn should_parse_sub_sub_stages() {
        let code = r#"pipeline {
    agent none
    stages {
        stage('Sequential') {
            agent {
                label 'for-sequential'
            }
            environment {
                FOR_SEQUENTIAL = "some-value"
            }
            stages {
                stage('In Sequential 1') {
                    steps {
                        echo "In Sequential 1"
                    }
                }
                stage('In Sequential 2') {
                    steps {
                        echo "In Sequential 2"
                    }
                }
                stage('Parallel In Sequential') {
                    parallel {
                        stage('In Parallel 1') {
                            steps {
                                echo "In Parallel 1"
                            }
                        }
                        stage('In Parallel 2') {
                            steps {
                                echo "In Parallel 2"
                            }
                        }
                    }
                }
            }
        }
    }
}
        "#;
        let jenkinsfile = Jenkinsfile::from_str(code).unwrap();

        assert_eq!(3, jenkinsfile.stages[0].sub_stages.len());
        assert_eq!(true, jenkinsfile.stages[0].sub_stages[2].is_parallel);
        assert_eq!(2, jenkinsfile.stages[0].sub_stages[2].sub_stages.len());
    }

    #[test]
    pub fn should_parse_multiple_stages() {
        let code = r#"pipeline {
    agent none
    stages {
        stage('Build') {
            agent any
            steps {
                checkout scm
                sh 'make'
                stash includes: '**/target/*.jar', name: 'app'
            }
        }
        stage('Test on Linux') {
            agent {
                label 'linux'
            }
            steps {
                unstash 'app'
                sh 'make check'
            }
            post {
                always {
                    junit '**/target/*.xml'
                }
            }
        }
        stage('Test on Windows') {
            agent {
                label 'windows'
            }
            steps {
                unstash 'app'
                bat 'make check'
            }
            post {
                always {
                    junit '**/target/*.xml'
                }
            }
        }
    }
}
        "#;
        let jenkinsfile = Jenkinsfile::from_str(code).unwrap();
        assert_eq!(3, jenkinsfile.stages.len());
        assert_eq!("Build", jenkinsfile.stages[0].name);
        assert_eq!(3, jenkinsfile.stages[0].steps.len());
    }
}