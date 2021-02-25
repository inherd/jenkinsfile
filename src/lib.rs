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

        while let Some(parsed) = parser.next() {
            match parsed.as_rule() {
                Rule::stagesDecl => {
                    self.parse_stages(&mut parsed.into_inner())?;
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
    fn parse_stage(&mut self, parser: &mut Pairs<Rule>) -> Result<(), PestError<Rule>> {
        let mut stage = JenkinsStage::default();
        while let Some(parsed) = parser.next() {
            match parsed.as_rule() {
                Rule::string => {
                    stage.name = self.parse_string(&mut parsed.into_inner());
                }
                Rule::stepsDecl => {
                    //
                }
                Rule::parallelDecl => {
                    //
                }
                Rule::stagesDecl => {
                   //
                    self.parse_stages(&mut parsed.into_inner())?;
                }
                _ => {}
            }
        }


        self.stages.push(stage);

        Ok(())
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
    fn parse_stages(&mut self, parser: &mut Pairs<Rule>) -> Result<(), PestError<Rule>> {
        while let Some(parsed) = parser.next() {
            match parsed.as_rule() {
                Rule::stage => {
                    self.parse_stage(&mut parsed.into_inner())?;
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

impl Default for JenkinsStage {
    fn default() -> Self {
        JenkinsStage {
            name: "".to_string(),
            jobs: vec![],
        }
    }
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
        let jenkinsfile = Jenkinsfile::from_str(code).unwrap();
        assert_eq!(1, jenkinsfile.stages.len());
        assert_eq!("build", jenkinsfile.stages[0].name);
    }
}