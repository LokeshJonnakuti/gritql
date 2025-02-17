use super::{
    patterns::{Matcher, Name, Pattern},
    resolved_pattern::ResolvedPattern,
    State,
};
use crate::{context::Context, resolve};
use anyhow::Result;
use im::vector;
use marzano_util::analysis_logs::AnalysisLogs;

#[derive(Debug, Clone)]
pub struct Every {
    pub pattern: Pattern,
}

impl Every {
    pub fn new(pattern: Pattern) -> Self {
        Self { pattern }
    }
}

impl Name for Every {
    fn name(&self) -> &'static str {
        "EVERY"
    }
}

impl Matcher for Every {
    fn execute<'a>(
        &'a self,
        binding: &ResolvedPattern<'a>,
        init_state: &mut State<'a>,
        context: &'a impl Context,
        logs: &mut AnalysisLogs,
    ) -> Result<bool> {
        // might be necessary to clone init state at the top,
        // but more performant to not, so leaving out for now.
        match binding {
            ResolvedPattern::Binding(bindings) => {
                let binding = resolve!(bindings.last());
                let Some(list_items) = binding.list_items() else {
                    return Ok(false);
                };

                for item in list_items {
                    if !self.pattern.execute(
                        &ResolvedPattern::from_node(item),
                        init_state,
                        context,
                        logs,
                    )? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            ResolvedPattern::List(elements) => {
                let pattern = &self.pattern;
                for element in elements {
                    if !pattern.execute(element, init_state, context, logs)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            ResolvedPattern::Map(map) => {
                let pattern = &self.pattern;
                for (key, value) in map {
                    let key =
                        ResolvedPattern::Constant(crate::binding::Constant::String(key.clone()));
                    let resolved = ResolvedPattern::List(vector![key, value.clone()]);
                    if !pattern.execute(&resolved, init_state, context, logs)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            ResolvedPattern::Snippets(_)
            | ResolvedPattern::File(_)
            | ResolvedPattern::Files(_)
            | ResolvedPattern::Constant(_) => Ok(false),
        }
    }
}
