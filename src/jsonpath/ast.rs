//! Abstract syntax tree types for JSONPath expressions.

/// A segment in a JSONPath expression.
#[derive(Debug, Clone, PartialEq)]
pub enum PathSegment {
    /// Root node ($)
    Root,
    /// Current node (@) - reserved for future filter support
    Current,
    /// Named child (.property or ['property'])
    Child(String),
    /// Array index ([0], [-1])
    Index(isize),
    /// Wildcard (* or [*]) - all children
    Wildcard,
    /// Recursive descent (.. or ..property)
    RecursiveDescent(Option<String>),
    /// Array slice ([start:end])
    Slice(Option<isize>, Option<isize>),
    /// Multiple properties (['prop1','prop2'])
    MultiProperty(Vec<String>),
}

/// A complete JSONPath expression.
#[derive(Debug, Clone, PartialEq)]
pub struct JsonPath {
    /// Segments that make up the path.
    pub segments: Vec<PathSegment>,
}

impl JsonPath {
    /// Creates a new JSONPath with the given segments.
    pub fn new(segments: Vec<PathSegment>) -> Self {
        Self { segments }
    }
}
