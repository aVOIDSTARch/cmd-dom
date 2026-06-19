use serde::{Deserialize, Serialize};

/// A shell-ready command string produced by the interactive builder.
///
/// This is the terminal output of a build session — it contains everything
/// needed to execute the command and to round-trip back to a format string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullCommand {
    /// The root command name, e.g. `"git"`.
    pub name: String,

    /// Human description of what this specific invocation does.
    pub description: String,

    /// The complete, shell-ready string, e.g. `"git commit -m \"initial\""`.
    pub cmd_string: String,

    /// Optionally derived format string for saving as a reusable template.
    pub format_string: Option<CmdFormatString>,
}

/// A reusable command template with named placeholders.
///
/// Stored in [`CommandMap::saved_format_strings`](crate::library::CommandMap)
/// so that frequently used invocations can be recalled and re-parameterized
/// without going through the full interactive builder.
///
/// ## Template syntax
///
/// Placeholders are `{name}`, e.g.:
/// ```text
/// git commit -m {message} --author {author}
/// docker run -p {host_port}:{container_port} {image}
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CmdFormatString {
    /// Short memorable alias for this template, e.g. `"git-commit"`.
    pub alias: String,

    /// One-sentence description of what the template achieves.
    pub description: String,

    /// The template string with `{placeholder}` tokens.
    pub template: String,
}

impl CmdFormatString {
    /// Render the template by substituting `{key}` tokens with the
    /// provided key-value pairs.  Returns an error listing any
    /// placeholders that were not supplied.
    pub fn render(&self, values: &[(&str, &str)]) -> Result<String, Vec<String>> {
        let mut result = self.template.clone();
        let mut missing: Vec<String> = Vec::new();

        // Collect all {placeholder} tokens present in the template
        let mut placeholders: Vec<String> = Vec::new();
        let mut chars = self.template.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '{' {
                let token: String = chars.by_ref().take_while(|&x| x != '}').collect();
                if !token.is_empty() {
                    placeholders.push(token);
                }
            }
        }

        for placeholder in &placeholders {
            let key = format!("{{{}}}", placeholder);
            if let Some((_, v)) = values.iter().find(|(k, _)| *k == placeholder.as_str()) {
                result = result.replace(&key, v);
            } else {
                missing.push(placeholder.clone());
            }
        }

        if missing.is_empty() {
            Ok(result)
        } else {
            Err(missing)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_substitutes_all_placeholders() {
        let fmt = CmdFormatString {
            alias: "git-commit".into(),
            description: "Quick commit".into(),
            template: "git commit -m {message} --author {author}".into(),
        };
        let result = fmt
            .render(&[("message", "initial commit"), ("author", "alice")])
            .unwrap();
        assert_eq!(result, "git commit -m initial commit --author alice");
    }

    #[test]
    fn render_reports_missing_placeholders() {
        let fmt = CmdFormatString {
            alias: "git-commit".into(),
            description: "Quick commit".into(),
            template: "git commit -m {message} --author {author}".into(),
        };
        let err = fmt.render(&[("message", "hi")]).unwrap_err();
        assert_eq!(err, vec!["author"]);
    }
}
