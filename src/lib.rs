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
        let mut agents = false;
        let mut stages = false;

        while let Some(parsed) = parser.next() {
            match parsed.as_rule() {
                Rule::agentDecl => {
                    if agents {
                        return Err(PestError::new_from_span(
                            ErrorVariant::CustomError {
                                message: "Cannot have two top-level `agent` directives".to_string(),
                            },
                            parsed.as_span(),
                        ));
                    }
                    agents = true;
                }
                Rule::stagesDecl => {
                    if stages {
                        return Err(PestError::new_from_span(
                            ErrorVariant::CustomError {
                                message: "Cannot have two top-level `stages` directives".to_string(),
                            },
                            parsed.as_span(),
                        ));
                    }
                    stages = true;
                    self.parse_stages(&mut parsed.into_inner())?;
                }
                _ => {}
            }
        }
        /*
         * Both agents and stages are required, the lack thereof is an error
         */
        if !agents || !stages {
            let error = PestError::new_from_pos(
                ErrorVariant::ParsingError {
                    positives: vec![],
                    negatives: vec![],
                },
                pest::Position::from_start(buffer),
            );
            return Err(error);
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
    fn parse_stage(&mut self, parser: &mut Pairs<Rule>, span: pest::Span) -> Result<(), PestError<Rule>> {
        let mut met_requirements = false;

        while let Some(parsed) = parser.next() {
            match parsed.as_rule() {
                Rule::stepsDecl => {
                    met_requirements = true;
                }
                Rule::parallelDecl => {
                    met_requirements = true;
                }
                Rule::stagesDecl => {
                    met_requirements = true;
                    self.parse_stages(&mut parsed.into_inner())?;
                }
                _ => {}
            }
        }

        if !met_requirements {
            Err(PestError::new_from_span(
                ErrorVariant::CustomError {
                    message: "A stage must have either steps{}, parallel{}, or nested stages {}"
                        .to_string(),
                },
                span,
            ))
        } else {
            Ok(())
        }
    }

    fn parse_stages(&mut self, parser: &mut Pairs<Rule>) -> Result<(), PestError<Rule>> {
        while let Some(parsed) = parser.next() {
            match parsed.as_rule() {
                Rule::stage => {
                    let span = parsed.as_span();
                    self.parse_stage(&mut parsed.into_inner(), span)?;
                }
                _ => {}
            }
        }
        Ok(())
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