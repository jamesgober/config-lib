//! # Resolving NOML within limits
//!
//! The noml crate's resolver copies the referenced value for every `${...}`
//! and reads every `include` itself, with no limit on either. Ten lines of
//! `s1 = "${s0}${s0}..."` grow a few hundred bytes of text into gigabytes,
//! and an include can name any file. This module puts config-lib in front of
//! the resolver:
//!
//! 1. Includes are read here, through [`IncludePolicy`] (allowed
//!    directories, regular files only, size limit), parsed, and put in place
//!    of the `include` node, so the resolver never reads a file.
//! 2. A copy of the tree in which every value that uses `${...}` is replaced
//!    by a marker is resolved first. That copy has no interpolation, so its
//!    size is bounded by the input.
//! 3. Each `${...}` is then looked up in that copy, following the same rules
//!    as the resolver (included file first, then the including document),
//!    and the size of what it would copy is added up. The document is
//!    refused before anything is expanded if the total, the number of
//!    references, or the length of a reference chain is over its limit.
//! 4. References inside an included file that name a value of that file are
//!    rewritten to their full path, because the included text now sits
//!    inside the including document.
//! 5. The resolver runs on the checked tree.
//!
//! Errors carry a line and column and fixed text: they never quote the
//! document, an included file, a path or a reference.

use super::noml_guard::{self, Policy};
use crate::error::{Error, Result};
use crate::load::{IncludePolicy, LoadOptions, MAX_INCLUDE_DEPTH};
use noml::parser::ast::{
    AstNode, AstValue, Comments, FormatMetadata, Key, KeySegment, Span, StringStyle, TableEntry,
};
use noml::{Document, Value as NValue};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Prefix of the string that stands in for a value using `${...}`
const REF_MARK: &str = "\u{0}config-lib-ref:";
/// Prefix of the key that marks where an included file's table sits
const INCLUDE_MARK: &str = "\u{0}config-lib-include:";
/// Size counted for every value, on top of its text
const VALUE_COST: u64 = 32;
/// Extra size counted for a native type built from interpolated arguments
const NATIVE_COST: u64 = 64;

/// One step in a path through a value tree
#[derive(Debug, Clone, PartialEq, Eq)]
enum Seg {
    Key(String),
    Index(usize),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// A double-quoted string containing `${`
    Template,
    /// A bare `${path}` value
    Bare,
    /// A function call or native type with interpolated arguments
    Call,
}

/// A value that uses `${...}`, in document order
struct Site {
    node: AstNode,
    kind: Kind,
    /// Include ids from the outermost include to the innermost
    chain: Vec<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Waiting,
    Active,
    Done,
}

/// Parse and resolve NOML text.
///
/// `file` is the path the text was read from, if any; includes resolve
/// against its directory.
pub(crate) fn resolve_source(
    source: &str,
    file: Option<&Path>,
    options: &LoadOptions,
) -> Result<NValue> {
    let document = parse(source)?;
    resolve_document(&document, file, options)
}

/// Parse NOML text into a document, with errors that do not quote it
pub(crate) fn parse(source: &str) -> Result<Document> {
    noml::parse_string(source, None).map_err(|e| noml_guard::noml_error(e, "NOML"))
}

/// Resolve a parsed document within the limits in `options`.
pub(crate) fn resolve_document(
    document: &Document,
    file: Option<&Path>,
    options: &LoadOptions,
) -> Result<NValue> {
    if options.strict {
        noml_guard::check(document, Policy::StrictNoml)?;
    }
    let policy = IncludePolicy::new(file, options);
    let mut prep = Prepare {
        policy: &policy,
        sites: Vec::new(),
        includes: 0,
        stack: Vec::new(),
    };
    let mut real = document.root.clone();
    let mut neutral = document.root.clone();
    let base_dir = policy.base_dir.clone();
    prep.walk(&mut real, &mut neutral, &[], &base_dir, false)?;

    let Prepare {
        sites, includes, ..
    } = prep;

    if !sites.is_empty() {
        let mut tree = run_resolver(neutral)?;
        let mut include_locs = vec![None; includes];
        collect_includes(&mut tree, &mut Vec::new(), &mut include_locs);
        let include_locs: Vec<Vec<Seg>> = include_locs
            .into_iter()
            .map(|loc| loc.unwrap_or_default())
            .collect();

        let mut measure = Measure::new(&tree, &sites, &include_locs, options);
        measure.run()?;
        if includes > 0 {
            let decisions = measure.decisions;
            let mut rewrite = Rewrite {
                sites: &sites,
                include_locs: &include_locs,
                decisions: &decisions,
                next: 0,
            };
            rewrite.walk(&mut real)?;
        }
    }
    run_resolver(real)
}

fn run_resolver(root: AstNode) -> Result<NValue> {
    let document = Document {
        root,
        source_path: None,
        source_text: None,
    };
    noml::Resolver::new()
        .resolve(&document)
        .map_err(|e| noml_guard::noml_error(e, "NOML"))
}

// =========================================================================
// Step 1: read includes, find the values that use ${...}
// =========================================================================

struct Prepare<'p> {
    policy: &'p IncludePolicy,
    sites: Vec<Site>,
    /// Number of includes read so far
    includes: usize,
    /// Canonical paths of the files being included, outermost first
    stack: Vec<PathBuf>,
}

fn at(span: Span, message: &str, in_include: bool) -> Error {
    let message = if in_include {
        format!("{message} (in an included file)")
    } else {
        message.to_string()
    };
    Error::parse(message, span.start_line, span.start_column)
}

fn marker_node(text: String, span: Span) -> AstNode {
    AstNode {
        value: AstValue::String {
            value: text,
            style: StringStyle::Single,
            has_escapes: false,
        },
        span,
        comments: Comments::default(),
        format: FormatMetadata::default(),
    }
}

impl Prepare<'_> {
    /// Walk `real` and its copy `neutral` together. Includes are read and
    /// put in place in both; in `neutral`, every value that uses `${...}` is
    /// replaced by a marker.
    fn walk(
        &mut self,
        real: &mut AstNode,
        neutral: &mut AstNode,
        chain: &[usize],
        dir: &Path,
        in_include: bool,
    ) -> Result<()> {
        let kind = match &real.value {
            AstValue::String { value, style, .. } if is_template(value, *style) => {
                Some(Kind::Template)
            }
            AstValue::Interpolation { .. } => Some(Kind::Bare),
            AstValue::FunctionCall { args, .. } | AstValue::Native { args, .. }
                if args.iter().any(has_template) =>
            {
                Some(Kind::Call)
            }
            _ => None,
        };
        if let Some(kind) = kind {
            let id = self.sites.len();
            self.sites.push(Site {
                node: real.clone(),
                kind,
                chain: chain.to_vec(),
            });
            *neutral = marker_node(format!("{REF_MARK}{id}"), real.span);
            return Ok(());
        }

        match (&mut real.value, &mut neutral.value) {
            (AstValue::Array { elements: r, .. }, AstValue::Array { elements: n, .. }) => {
                for (r, n) in r.iter_mut().zip(n.iter_mut()) {
                    self.walk(r, n, chain, dir, in_include)?;
                }
            }
            (AstValue::Table { entries: r, .. }, AstValue::Table { entries: n, .. }) => {
                for (r, n) in r.iter_mut().zip(n.iter_mut()) {
                    self.walk(&mut r.value, &mut n.value, chain, dir, in_include)?;
                }
            }
            (AstValue::Include { path }, _) => {
                let path = path.clone();
                let (mut inc_real, mut inc_neutral, id) =
                    self.include(&path, real.span, chain, dir, in_include)?;
                if let AstValue::Table { entries, .. } = &mut inc_neutral.value {
                    entries.push(include_mark(id, real.span));
                }
                // The included file's own includes were read in `include`
                std::mem::swap(real, &mut inc_real);
                std::mem::swap(neutral, &mut inc_neutral);
            }
            _ => {}
        }
        Ok(())
    }

    /// Read, parse and prepare one included file
    fn include(
        &mut self,
        include: &str,
        span: Span,
        chain: &[usize],
        dir: &Path,
        in_include: bool,
    ) -> Result<(AstNode, AstNode, usize)> {
        if self.stack.len() >= MAX_INCLUDE_DEPTH {
            return Err(at(span, "includes are nested too deeply", in_include));
        }
        if self.includes >= self.policy.max_includes {
            return Err(at(span, "too many included files", in_include));
        }
        let (canonical, text) = self
            .policy
            .read(dir, include)
            .map_err(|why| at(span, why, in_include))?;
        if self.stack.contains(&canonical) {
            return Err(at(span, "a file includes itself", in_include));
        }
        let document = parse(&text).map_err(|e| e.in_included_file())?;
        let mut inc_real = document.root;
        if !matches!(inc_real.value, AstValue::Table { .. }) {
            return Err(at(span, "an included file must hold a table", in_include));
        }
        let mut inc_neutral = inc_real.clone();
        let id = self.includes;
        self.includes += 1;

        let mut inner = chain.to_vec();
        inner.push(id);
        let inc_dir = canonical
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        self.stack.push(canonical);
        let result = self.walk(&mut inc_real, &mut inc_neutral, &inner, &inc_dir, true);
        self.stack.pop();
        result?;
        Ok((inc_real, inc_neutral, id))
    }
}

/// The table entry that marks where include `id` sits
fn include_mark(id: usize, span: Span) -> TableEntry {
    TableEntry {
        key: Key {
            segments: vec![KeySegment {
                name: format!("{INCLUDE_MARK}{id}"),
                quoted: true,
                quote_style: Some(StringStyle::Double),
            }],
            span,
        },
        value: AstNode {
            value: AstValue::Bool(true),
            span,
            comments: Comments::default(),
            format: FormatMetadata::default(),
        },
        comments: Comments::default(),
    }
}

/// Remove the include marks from a resolved tree, recording where each was
fn collect_includes(value: &mut NValue, path: &mut Vec<Seg>, locs: &mut [Option<Vec<Seg>>]) {
    match value {
        NValue::Table(table) => {
            let marks: Vec<String> = table
                .keys()
                .filter(|k| k.starts_with(INCLUDE_MARK))
                .cloned()
                .collect();
            for mark in marks {
                table.remove(&mark);
                let id = mark[INCLUDE_MARK.len()..].parse::<usize>().ok();
                if let Some(slot) = id.and_then(|id| locs.get_mut(id)) {
                    *slot = Some(path.clone());
                }
            }
            for (key, child) in table.iter_mut() {
                path.push(Seg::Key(key.clone()));
                collect_includes(child, path, locs);
                path.pop();
            }
        }
        NValue::Array(items) => {
            for (i, child) in items.iter_mut().enumerate() {
                path.push(Seg::Index(i));
                collect_includes(child, path, locs);
                path.pop();
            }
        }
        _ => {}
    }
}

/// True if a string value needs interpolation (as in the noml resolver)
fn is_template(value: &str, style: StringStyle) -> bool {
    matches!(style, StringStyle::Double | StringStyle::TripleDouble) && value.contains("${")
}

/// True if a node or anything inside it needs interpolation
fn has_template(node: &AstNode) -> bool {
    match &node.value {
        AstValue::String { value, style, .. } => is_template(value, *style),
        AstValue::Interpolation { .. } => true,
        AstValue::Array { elements, .. } => elements.iter().any(has_template),
        AstValue::Table { entries, .. } => entries.iter().any(|e| has_template(&e.value)),
        AstValue::FunctionCall { args, .. } | AstValue::Native { args, .. } => {
            args.iter().any(has_template)
        }
        _ => false,
    }
}

/// Split a reference into its segments, as the noml resolver does
fn parse_reference(expression: &str) -> Option<Vec<String>> {
    let mut names = Vec::new();
    let mut rest = expression.trim();
    if rest.is_empty() {
        return None;
    }
    loop {
        rest = rest.trim_start();
        let (name, after) = if let Some(quoted) = rest.strip_prefix('"') {
            let end = quoted.find('"')?;
            (quoted[..end].to_string(), &quoted[end + 1..])
        } else {
            let end = rest.find(['.', '[']).unwrap_or(rest.len());
            let name = rest[..end].trim();
            if name.is_empty() {
                return None;
            }
            (name.to_string(), &rest[end..])
        };
        names.push(name);
        rest = after.trim_start();
        while let Some(inner) = rest.strip_prefix('[') {
            let end = inner.find(']')?;
            let index = inner[..end].trim();
            if index.is_empty() || !index.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            names.push(index.to_string());
            rest = inner[end + 1..].trim_start();
        }
        if rest.is_empty() {
            return Some(names);
        }
        rest = rest.strip_prefix('.')?;
    }
}

/// The `${...}` expressions of a template string, in order. `$${` is a
/// literal `${`. Each item is the byte range of the expression.
fn template_refs(text: &str) -> Vec<std::ops::Range<usize>> {
    let mut refs = Vec::new();
    let mut pos = 0;
    while let Some(found) = text[pos..].find('$') {
        let start = pos + found;
        let tail = &text[start..];
        if tail.starts_with("$${") {
            pos = start + 3;
        } else if tail.starts_with("${") {
            match tail.find('}') {
                Some(close) => {
                    refs.push(start + 2..start + close);
                    pos = start + close + 1;
                }
                // Unclosed: the resolver reports it
                None => break,
            }
        } else {
            pos = start + 1;
        }
    }
    refs
}

// =========================================================================
// Step 3: add up what every ${...} would copy
// =========================================================================

struct Measure<'t> {
    tree: &'t NValue,
    sites: &'t [Site],
    include_locs: &'t [Vec<Seg>],
    size_state: Vec<State>,
    sizes: Vec<u64>,
    target_state: Vec<State>,
    targets: Vec<Option<&'t NValue>>,
    /// For each site, the scope each of its references was found in
    decisions: Vec<Vec<usize>>,
    /// Size of each container already measured, by address
    value_sizes: HashMap<usize, u64>,
    references: usize,
    /// Longest chain of references starting at each site, and at each bare
    /// site's target lookup
    chains: Vec<usize>,
    target_chains: Vec<usize>,
    /// Longest chain found so far below each site being measured
    open_chains: Vec<usize>,
    max_size: u64,
    max_references: usize,
    max_depth: usize,
    env_max: Option<u64>,
}

impl<'t> Measure<'t> {
    fn new(
        tree: &'t NValue,
        sites: &'t [Site],
        include_locs: &'t [Vec<Seg>],
        options: &LoadOptions,
    ) -> Self {
        let n = sites.len();
        Self {
            tree,
            sites,
            include_locs,
            size_state: vec![State::Waiting; n],
            sizes: vec![0; n],
            target_state: vec![State::Waiting; n],
            targets: vec![None; n],
            decisions: vec![Vec::new(); n],
            value_sizes: HashMap::new(),
            references: 0,
            chains: vec![0; n],
            target_chains: vec![0; n],
            open_chains: Vec::new(),
            max_size: options.max_expanded_size,
            max_references: options.max_expansions,
            max_depth: options.max_expansion_depth,
            env_max: None,
        }
    }

    fn run(&mut self) -> Result<()> {
        let total = self.value_size(self.tree, 0)?;
        if total > self.max_size {
            return Err(Error::parse(
                "the document expands past the size limit through ${...} references",
                1,
                1,
            ));
        }
        // Every site is reached from the root, but make sure each was
        // looked at so that every reference has a decision
        for k in 0..self.sites.len() {
            self.site_size(k, 0)?;
        }
        Ok(())
    }

    /// Recursion guard for the measuring code itself. The chain limit is
    /// checked exactly in [`Measure::close_chain`]; this only keeps the
    /// stack bounded on the way there.
    fn guard(&self, k: usize, depth: usize) -> Result<()> {
        if depth > self.max_depth.saturating_mul(4).saturating_add(8) {
            return Err(self.error(k, "a chain of ${...} references is too long"));
        }
        Ok(())
    }

    /// Count a finished chain of length `len` in the site being measured
    fn bump_chain(&mut self, len: usize) {
        if let Some(open) = self.open_chains.last_mut() {
            *open = (*open).max(len);
        }
    }

    /// Finish the chain of the site being measured, refuse it if it is too
    /// long, and count it in the site above. The longest chain is what
    /// bounds the resolver's recursion, whatever order it settles values in.
    fn close_chain(&mut self, k: usize) -> Result<usize> {
        let len = self.open_chains.pop().unwrap_or(0) + 1;
        if len > self.max_depth {
            return Err(self.error(k, "a chain of ${...} references is too long"));
        }
        self.bump_chain(len);
        Ok(len)
    }

    fn error(&self, k: usize, message: &str) -> Error {
        let site = &self.sites[k];
        at(site.node.span, message, !site.chain.is_empty())
    }

    /// Lookup scopes of site `k`: innermost include first, the root last
    fn scopes(&self, k: usize) -> Vec<&'t [Seg]> {
        let include_locs = self.include_locs;
        let mut scopes: Vec<&'t [Seg]> = self.sites[k]
            .chain
            .iter()
            .rev()
            .map(|&id| include_locs.get(id).map_or(&[][..], |loc| &loc[..]))
            .collect();
        scopes.push(&[]);
        scopes
    }

    fn marker(value: &NValue) -> Option<usize> {
        match value {
            NValue::String(s) => s.strip_prefix(REF_MARK)?.parse().ok(),
            _ => None,
        }
    }

    /// Follow a bare `${...}` marker to the value it copies
    fn deref(&mut self, mut value: &'t NValue, depth: usize) -> Result<&'t NValue> {
        let mut hops = 0;
        while let Some(m) = Self::marker(value) {
            if self.sites.get(m).map(|s| s.kind) != Some(Kind::Bare) {
                break;
            }
            value = self.target(m, depth + hops)?;
            hops += 1;
        }
        Ok(value)
    }

    /// The value the bare site `k` copies
    fn target(&mut self, k: usize, depth: usize) -> Result<&'t NValue> {
        self.guard(k, depth)?;
        match self.target_state[k] {
            State::Done => {
                if let Some(value) = self.targets[k] {
                    self.bump_chain(self.target_chains[k]);
                    return Ok(value);
                }
            }
            State::Active => return Err(self.error(k, "a ${...} reference refers to itself")),
            State::Waiting => {}
        }
        self.target_state[k] = State::Active;
        self.open_chains.push(0);
        let path = match &self.sites[k].node.value {
            AstValue::Interpolation { path } => path.clone(),
            _ => String::new(),
        };
        let value = self.lookup_ref(k, &path, depth)?;
        let value = self.deref(value, depth + 1)?;
        self.target_chains[k] = self.close_chain(k)?;
        self.targets[k] = Some(value);
        self.target_state[k] = State::Done;
        Ok(value)
    }

    /// Look up one reference of site `k` and record the scope it was found in
    fn lookup_ref(&mut self, k: usize, expression: &str, depth: usize) -> Result<&'t NValue> {
        self.references += 1;
        if self.references > self.max_references {
            return Err(self.error(k, "the document has too many ${...} references"));
        }
        let names = parse_reference(expression)
            .ok_or_else(|| self.error(k, "a ${...} reference is not a valid path"))?;
        for (index, scope) in self.scopes(k).into_iter().enumerate() {
            if let Some(value) = self.find(scope, &names, depth)? {
                self.decisions[k].push(index);
                return Ok(value);
            }
        }
        Err(self.error(k, "a ${...} reference names a value that does not exist"))
    }

    /// Walk `names` from `scope`, following bare references on the way
    fn find(
        &mut self,
        scope: &[Seg],
        names: &[String],
        depth: usize,
    ) -> Result<Option<&'t NValue>> {
        let mut current = self.tree;
        for seg in scope {
            current = match (current, seg) {
                (NValue::Table(t), Seg::Key(k)) => match t.get(k) {
                    Some(v) => v,
                    None => return Ok(None),
                },
                (NValue::Array(a), Seg::Index(i)) => match a.get(*i) {
                    Some(v) => v,
                    None => return Ok(None),
                },
                _ => return Ok(None),
            };
        }
        for name in names {
            current = self.deref(current, depth + 1)?;
            let next = match current {
                NValue::Table(t) => t.get(name),
                NValue::Array(a) => name.parse::<usize>().ok().and_then(|i| a.get(i)),
                _ => None,
            };
            match next {
                Some(v) => current = v,
                None => return Ok(None),
            }
        }
        Ok(Some(current))
    }

    /// Expanded size of site `k`
    fn site_size(&mut self, k: usize, depth: usize) -> Result<u64> {
        self.guard(k, depth)?;
        match self.size_state[k] {
            State::Done => {
                self.bump_chain(self.chains[k]);
                return Ok(self.sizes[k]);
            }
            State::Active => return Err(self.error(k, "a ${...} reference refers to itself")),
            State::Waiting => {}
        }
        self.size_state[k] = State::Active;
        self.open_chains.push(0);
        let size = match self.sites[k].kind {
            Kind::Bare => {
                let target = self.target(k, depth)?;
                self.value_size(target, depth + 1)?
            }
            Kind::Template | Kind::Call => {
                let sites = self.sites;
                self.node_size(&sites[k].node, k, depth)?
            }
        };
        if size > self.max_size {
            return Err(self.error(k, "a ${...} reference expands past the size limit"));
        }
        self.chains[k] = self.close_chain(k)?;
        self.sizes[k] = size;
        self.size_state[k] = State::Done;
        Ok(size)
    }

    /// Size of an AST node of site `k` once its references are expanded.
    /// References are visited in document order, matching [`Rewrite`].
    fn node_size(&mut self, node: &AstNode, k: usize, depth: usize) -> Result<u64> {
        Ok(match &node.value {
            AstValue::String { value, style, .. } if is_template(value, *style) => {
                let mut size = VALUE_COST + value.len() as u64;
                for range in template_refs(value) {
                    let target = self.lookup_ref(k, &value[range], depth)?;
                    let target = self.deref(target, depth + 1)?;
                    size = size.saturating_add(self.text_size(target, depth + 1)?);
                }
                size
            }
            AstValue::String { value, .. } => VALUE_COST + value.len() as u64,
            AstValue::Interpolation { path } => {
                let target = self.lookup_ref(k, path, depth)?;
                let target = self.deref(target, depth + 1)?;
                self.value_size(target, depth + 1)?
            }
            AstValue::Array { elements, .. } => {
                let mut size = VALUE_COST;
                for element in elements {
                    size = size.saturating_add(self.node_size(element, k, depth)?);
                }
                size
            }
            AstValue::Table { entries, .. } => {
                let mut size = VALUE_COST;
                for entry in entries {
                    let key: u64 = entry.key.segments.iter().map(|s| s.name.len() as u64).sum();
                    size = size.saturating_add(key).saturating_add(self.node_size(
                        &entry.value,
                        k,
                        depth,
                    )?);
                }
                size
            }
            AstValue::FunctionCall { name, args } => {
                let mut size = VALUE_COST;
                if name == "env" {
                    size = size.saturating_add(self.env_max());
                }
                for arg in args {
                    size = size.saturating_add(self.node_size(arg, k, depth)?);
                }
                size
            }
            AstValue::Native { args, .. } => {
                let mut size = VALUE_COST + NATIVE_COST;
                for arg in args {
                    size = size.saturating_add(self.node_size(arg, k, depth)?);
                }
                size
            }
            _ => VALUE_COST,
        })
    }

    /// Size of a value written into a string
    fn text_size(&mut self, value: &'t NValue, depth: usize) -> Result<u64> {
        if let Some(m) = Self::marker(value) {
            return self.site_size(m, depth);
        }
        match value {
            NValue::String(s) => Ok(s.len() as u64),
            // The resolver refuses tables and arrays in strings; count them
            // anyway, so the estimate is never low
            NValue::Table(_) | NValue::Array(_) => self.value_size(value, depth),
            _ => Ok(VALUE_COST),
        }
    }

    /// Expanded size of a value of the tree. Containers are walked with an
    /// explicit stack and remembered by address, so each is measured once.
    fn value_size(&mut self, root: &'t NValue, depth: usize) -> Result<u64> {
        let addr = |v: &NValue| v as *const NValue as usize;
        if let Some(&size) = self.value_sizes.get(&addr(root)) {
            return Ok(size);
        }
        let mut stack: Vec<(&'t NValue, bool)> = vec![(root, false)];
        let mut last = 0;
        while let Some((value, children_done)) = stack.pop() {
            if let Some(&size) = self.value_sizes.get(&addr(value)) {
                last = size;
                continue;
            }
            let size = match value {
                NValue::Table(table) if !children_done => {
                    stack.push((value, true));
                    stack.extend(table.values().map(|v| (v, false)));
                    continue;
                }
                NValue::Array(items) if !children_done => {
                    stack.push((value, true));
                    stack.extend(items.iter().map(|v| (v, false)));
                    continue;
                }
                NValue::Table(table) => {
                    let mut size = VALUE_COST;
                    for (key, child) in table {
                        let child = self.value_sizes.get(&addr(child)).copied().unwrap_or(0);
                        size = size.saturating_add(key.len() as u64).saturating_add(child);
                    }
                    size
                }
                NValue::Array(items) => {
                    let mut size = VALUE_COST;
                    for child in items {
                        let child = self.value_sizes.get(&addr(child)).copied().unwrap_or(0);
                        size = size.saturating_add(child);
                    }
                    size
                }
                NValue::String(s) => match Self::marker(value) {
                    Some(m) => self.site_size(m, depth)?,
                    None => VALUE_COST + s.len() as u64,
                },
                _ => VALUE_COST,
            };
            if size > self.max_size {
                return Err(Error::parse(
                    "the document expands past the size limit through ${...} references",
                    1,
                    1,
                ));
            }
            self.value_sizes.insert(addr(value), size);
            last = size;
        }
        Ok(last)
    }

    /// Longest environment variable value, counted for `env()` with an
    /// interpolated name
    fn env_max(&mut self) -> u64 {
        *self.env_max.get_or_insert_with(|| {
            std::env::vars_os()
                .map(|(_, v)| v.len() as u64)
                .max()
                .unwrap_or(0)
        })
    }
}

// =========================================================================
// Step 4: rewrite references in included files to full paths
// =========================================================================

struct Rewrite<'r> {
    sites: &'r [Site],
    include_locs: &'r [Vec<Seg>],
    decisions: &'r [Vec<usize>],
    next: usize,
}

impl Rewrite<'_> {
    /// Visit the tree in the order [`Prepare::walk`] did
    fn walk(&mut self, node: &mut AstNode) -> Result<()> {
        let is_site = match &node.value {
            AstValue::String { value, style, .. } => is_template(value, *style),
            AstValue::Interpolation { .. } => true,
            AstValue::FunctionCall { args, .. } | AstValue::Native { args, .. } => {
                args.iter().any(has_template)
            }
            _ => false,
        };
        if is_site {
            let k = self.next;
            self.next += 1;
            let mut index = 0;
            return self.rewrite(node, k, &mut index);
        }
        match &mut node.value {
            AstValue::Array { elements, .. } => {
                for element in elements {
                    self.walk(element)?;
                }
            }
            AstValue::Table { entries, .. } => {
                for entry in entries {
                    self.walk(&mut entry.value)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// The prefix that turns reference `index` of site `k` into a full
    /// path, or `None` when it was found from the document root
    fn prefix(&self, k: usize, index: usize, template: bool) -> Result<Option<String>> {
        let site = &self.sites[k];
        let scope = self
            .decisions
            .get(k)
            .and_then(|d| d.get(index))
            .copied()
            .unwrap_or(site.chain.len());
        if scope >= site.chain.len() {
            return Ok(None);
        }
        let id = site.chain[site.chain.len() - 1 - scope];
        let loc = self.include_locs.get(id).map_or(&[][..], |l| &l[..]);
        if loc.is_empty() {
            return Ok(None);
        }
        let mut prefix = String::new();
        for seg in loc {
            match seg {
                Seg::Key(name) => {
                    if !prefix.is_empty() {
                        prefix.push('.');
                    }
                    let bare = !name.is_empty()
                        && name.trim() == name
                        && !name.contains(['.', '[', '"', '}'])
                        && !name.starts_with('"');
                    if bare {
                        prefix.push_str(name);
                    } else if !(name.contains('"') || (template && name.contains('}'))) {
                        prefix.push('"');
                        prefix.push_str(name);
                        prefix.push('"');
                    } else {
                        return Err(at(
                            site.node.span,
                            "a ${...} reference in an included file cannot be resolved here",
                            true,
                        ));
                    }
                }
                Seg::Index(i) => {
                    prefix.push('[');
                    prefix.push_str(&i.to_string());
                    prefix.push(']');
                }
            }
        }
        Ok(Some(prefix))
    }

    fn rewrite(&mut self, node: &mut AstNode, k: usize, index: &mut usize) -> Result<()> {
        match &mut node.value {
            AstValue::String { value, style, .. } if is_template(value, *style) => {
                let text = value.clone();
                let mut out = String::with_capacity(text.len());
                let mut last = 0;
                for range in template_refs(&text) {
                    out.push_str(&text[last..range.start]);
                    let expression = &text[range.clone()];
                    match self.prefix(k, *index, true)? {
                        Some(prefix) => {
                            out.push_str(&prefix);
                            out.push('.');
                            out.push_str(expression.trim());
                        }
                        None => out.push_str(expression),
                    }
                    *index += 1;
                    last = range.end;
                }
                out.push_str(&text[last..]);
                *value = out;
            }
            AstValue::Interpolation { path } => {
                if let Some(prefix) = self.prefix(k, *index, false)? {
                    *path = format!("{prefix}.{}", path.trim());
                }
                *index += 1;
            }
            AstValue::Array { elements, .. } => {
                for element in elements {
                    self.rewrite(element, k, index)?;
                }
            }
            AstValue::Table { entries, .. } => {
                for entry in entries {
                    self.rewrite(&mut entry.value, k, index)?;
                }
            }
            AstValue::FunctionCall { args, .. } | AstValue::Native { args, .. } => {
                for arg in args {
                    self.rewrite(arg, k, index)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}
