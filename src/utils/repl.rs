use anyhow::Result;
use colored::*;
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::Highlighter;
use rustyline::hint::Hinter;
use rustyline::validate::Validator;
use rustyline::{Context, Editor, Helper};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

pub struct Repl<R>
where
    R: ReplRunner,
{
    runner: R,
    options: ReplOptions,
}

#[derive(Debug, Clone)]
pub struct ReplOptions {
    pub history_enabled: bool,
    pub history_path: PathBuf,
    pub max_history_lines: usize,
    pub completion_candidates: Vec<String>,
}

impl Default for ReplOptions {
    fn default() -> Self {
        let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push(".starforge");
        path.push("history");
        Self {
            history_enabled: true,
            history_path: path,
            max_history_lines: 1000,
            completion_candidates: Vec::new(),
        }
    }
}

pub trait ReplRunner {
    fn run_invocation(&mut self, function: &str, args: &[String]) -> Result<String>;
}

impl<R> Repl<R>
where
    R: ReplRunner,
{
    pub fn new(runner: R) -> Self {
        Self {
            runner,
            options: ReplOptions::default(),
        }
    }

    pub fn with_options(runner: R, options: ReplOptions) -> Self {
        Self { runner, options }
    }

    pub fn run(mut self) -> Result<()> {
        println!(
            "  {} {}",
            "StarForge Shell".bright_cyan().bold(),
            "(type :help for commands)".dimmed()
        );

        let mut editor = Editor::<StarForgeHelper, rustyline::history::DefaultHistory>::new()?;
        editor.set_helper(Some(StarForgeHelper::new(
            self.options.completion_candidates.clone(),
        )));
        self.load_history(&mut editor)?;

        loop {
            let prompt = format!("{}", "> ".bright_green().bold());
            let line = match editor.readline(&prompt) {
                Ok(line) => line.trim().to_string(),
                Err(ReadlineError::Interrupted) => continue,
                Err(ReadlineError::Eof) => break,
                Err(err) => return Err(err.into()),
            };
            if line.is_empty() {
                continue;
            }

            if line == ":q" || line == ":quit" || line == ":exit" {
                break;
            }

            if line == ":help" {
                println!("  {}", "Commands:".bold());
                println!("    :help              Show help");
                println!("    :quit | :exit      Exit shell");
                println!("    <TAB>              Complete wallet names and known contract IDs");
                println!("    fn(arg1, arg2)     Invoke a contract function");
                continue;
            }

            self.push_history(&mut editor, &line)?;
            let (function, args) = parse_invocation(&line)?;
            match self.runner.run_invocation(&function, &args) {
                Ok(out) => println!("{}", out),
                Err(e) => eprintln!("  {} {}", "✗".red().bold(), e),
            }
        }

        self.save_history(&mut editor)?;
        Ok(())
    }

    fn load_history(
        &self,
        editor: &mut Editor<StarForgeHelper, rustyline::history::DefaultHistory>,
    ) -> Result<()> {
        if !self.options.history_enabled {
            return Ok(());
        }

        match editor.load_history(&self.options.history_path) {
            Ok(()) => Ok(()),
            Err(ReadlineError::Io(e)) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    fn save_history(
        &self,
        editor: &mut Editor<StarForgeHelper, rustyline::history::DefaultHistory>,
    ) -> Result<()> {
        if !self.options.history_enabled {
            return Ok(());
        }

        if let Some(parent) = self.options.history_path.parent() {
            fs::create_dir_all(parent)?;
        }

        editor.save_history(&self.options.history_path)?;
        trim_history_file(&self.options.history_path, self.options.max_history_lines)?;
        Ok(())
    }

    fn push_history(
        &self,
        editor: &mut Editor<StarForgeHelper, rustyline::history::DefaultHistory>,
        line: &str,
    ) -> Result<()> {
        if !self.options.history_enabled {
            return Ok(());
        }
        editor.add_history_entry(line)?;
        Ok(())
    }
}

fn trim_history_file(path: &PathBuf, max_lines: usize) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let mut lines = content.lines().map(str::to_string).collect::<Vec<_>>();
    if max_lines == 0 {
        lines.clear();
    } else if lines.len() > max_lines {
        lines = lines.split_off(lines.len() - max_lines);
    }
    fs::write(
        path,
        lines.join("\n") + if lines.is_empty() { "" } else { "\n" },
    )?;
    Ok(())
}

#[derive(Clone, Debug)]
struct StarForgeHelper {
    candidates: Vec<String>,
}

impl StarForgeHelper {
    fn new(candidates: Vec<String>) -> Self {
        let mut seen = HashSet::new();
        let mut candidates = candidates
            .into_iter()
            .filter(|candidate| !candidate.trim().is_empty())
            .filter(|candidate| seen.insert(candidate.clone()))
            .collect::<Vec<_>>();
        candidates.sort();
        Self { candidates }
    }
}

impl Helper for StarForgeHelper {}
impl Hinter for StarForgeHelper {
    type Hint = String;
}
impl Highlighter for StarForgeHelper {}
impl Validator for StarForgeHelper {}

impl Completer for StarForgeHelper {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {
        let start = line[..pos]
            .rfind(|ch: char| ch.is_whitespace() || matches!(ch, '(' | ',' | '"' | '\''))
            .map(|idx| idx + 1)
            .unwrap_or(0);
        let prefix = &line[start..pos];
        let matches = self
            .candidates
            .iter()
            .filter(|candidate| candidate.starts_with(prefix))
            .map(|candidate| Pair {
                display: candidate.clone(),
                replacement: candidate.clone(),
            })
            .collect();
        Ok((start, matches))
    }
}

fn parse_invocation(input: &str) -> Result<(String, Vec<String>)> {
    let open = input
        .find('(')
        .ok_or_else(|| anyhow::anyhow!("Expected invocation like fn(\"arg\")"))?;
    let close = input
        .rfind(')')
        .ok_or_else(|| anyhow::anyhow!("Missing closing ')'"))?;
    if close < open {
        anyhow::bail!("Invalid invocation");
    }

    let function = input[..open].trim();
    if function.is_empty() {
        anyhow::bail!("Missing function name");
    }

    let args_raw = input[open + 1..close].trim();
    let args = split_args(args_raw)?;
    Ok((function.to_string(), args))
}

fn split_args(input: &str) -> Result<Vec<String>> {
    if input.is_empty() {
        return Ok(Vec::new());
    }

    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = '\0';
    let mut escape = false;

    for ch in input.chars() {
        if escape {
            current.push(ch);
            escape = false;
            continue;
        }

        if ch == '\\' {
            escape = true;
            continue;
        }

        if in_quotes {
            if ch == quote_char {
                in_quotes = false;
                continue;
            }
            current.push(ch);
            continue;
        }

        if ch == '"' || ch == '\'' {
            in_quotes = true;
            quote_char = ch;
            continue;
        }

        if ch == ',' {
            args.push(current.trim().to_string());
            current.clear();
            continue;
        }

        current.push(ch);
    }

    if in_quotes {
        anyhow::bail!("Unclosed quote in arguments");
    }

    if escape {
        anyhow::bail!("Trailing escape in arguments");
    }

    args.push(current.trim().to_string());
    Ok(args)
}
